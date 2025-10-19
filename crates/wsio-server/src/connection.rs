use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use bson::oid::ObjectId;
use dashmap::DashMap;
use http::HeaderMap;
use serde::de::DeserializeOwned;
use tokio::{
    spawn,
    sync::{
        Mutex,
        RwLock,
        mpsc::{
            UnboundedReceiver,
            UnboundedSender,
            unbounded_channel,
        },
    },
    task::JoinHandle,
    time::sleep,
};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    WsIoServer,
    core::packet::{
        WsIoPacket,
        WsIoPacketType,
    },
    namespace::WsIoServerNamespace,
    types::handler::{
        WsIoServerConnectionEventHandler,
        WsIoServerConnectionOnDisconnectHandler,
    },
};

enum WsIoServerConnectionStatus {
    Activating,
    AwaitingAuth,
    Closed,
    Closing,
    Created,
    Ready,
}

pub struct WsIoServerConnection {
    auth_timeout_task: Mutex<Option<JoinHandle<()>>>,
    event_handlers: DashMap<String, WsIoServerConnectionEventHandler>,
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    on_disconnect_handler: Mutex<Option<WsIoServerConnectionOnDisconnectHandler>>,
    sid: String,
    status: RwLock<WsIoServerConnectionStatus>,
    tx: UnboundedSender<Message>,
}

impl WsIoServerConnection {
    pub(crate) fn new(headers: HeaderMap, namespace: Arc<WsIoServerNamespace>) -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                auth_timeout_task: Mutex::new(None),
                event_handlers: DashMap::new(),
                headers,
                namespace,
                on_disconnect_handler: Mutex::new(None),
                sid: ObjectId::new().to_string(),
                status: RwLock::new(WsIoServerConnectionStatus::Created),
                tx,
            },
            rx,
        )
    }

    // Protected methods
    async fn abort_auth_timeout_task(&self) {
        if let Some(auth_timeout_task) = self.auth_timeout_task.lock().await.take() {
            auth_timeout_task.abort();
        }
    }

    async fn activate(self: &Arc<Self>) -> Result<()> {
        *self.status.write().await = WsIoServerConnectionStatus::Activating;
        if let Some(middleware) = &self.namespace.config.middleware {
            middleware(self.clone()).await?;
        }

        self.namespace.insert_connection(self.clone());
        *self.status.write().await = WsIoServerConnectionStatus::Ready;
        let packet = WsIoPacket {
            data: None,
            key: None,
            r#type: WsIoPacketType::Ready,
        };

        self.send_packet(&packet)?;
        (self.namespace.config.on_connect_handler)(self.clone()).await
    }

    pub(crate) async fn cleanup(self: &Arc<Self>) {
        *self.status.write().await = WsIoServerConnectionStatus::Closing;
        self.abort_auth_timeout_task().await;
        self.namespace.cleanup_connection(&self.sid);
        if let Some(on_disconnect_handler) = self.on_disconnect_handler.lock().await.take() {
            let _ = on_disconnect_handler(self.clone()).await;
        }

        *self.status.write().await = WsIoServerConnectionStatus::Closed;
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) {
        let packet = match self.namespace.config.packet_codec.decode(bytes) {
            Ok(packet) => packet,
            Err(_) => return,
        };

        match packet.r#type {
            WsIoPacketType::Auth => {
                if let Some(auth_handler) = &self.namespace.config.auth_handler
                    && (auth_handler)(self.clone(), packet.data.as_deref()).await.is_ok()
                {
                    self.abort_auth_timeout_task().await;
                    if self.activate().await.is_ok() {
                        return;
                    }
                }

                self.close();
            }
            WsIoPacketType::Event => {}
            _ => {}
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        let require_auth = self.namespace.config.auth_handler.is_some();
        let packet = WsIoPacket {
            data: Some(self.namespace.config.packet_codec.encode_data(&require_auth)?),
            key: Some(self.sid.clone()),
            r#type: WsIoPacketType::Init,
        };

        if require_auth {
            *self.status.write().await = WsIoServerConnectionStatus::AwaitingAuth;
            let connection = self.clone();
            self.auth_timeout_task.lock().await.replace(spawn(async move {
                sleep(connection.namespace.config.auth_timeout).await;
                if matches!(
                    *connection.status.read().await,
                    WsIoServerConnectionStatus::AwaitingAuth
                ) {
                    connection.close();
                }
            }));

            self.send_packet(&packet)?;
        } else {
            self.send_packet(&packet)?;
            self.activate().await?;
        }

        Ok(())
    }

    #[inline]
    fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        Ok(self.tx.send(self.namespace.encode_packet_to_message(packet)?)?)
    }

    // Public methods

    #[inline]
    pub fn close(&self) {
        let _ = self.tx.send(Message::Close(None));
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn namespace(&self) -> Arc<WsIoServerNamespace> {
        self.namespace.clone()
    }

    pub fn on<H, Fut, D>(&self, event: impl AsRef<str>, handler: H) -> Result<()>
    where
        H: Fn(Arc<WsIoServerConnection>, &D) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + 'static,
    {
        let event = event.as_ref();
        if self.event_handlers.contains_key(event) {
            bail!("Event {} handler already exists", event);
        }

        let handler = Arc::new(handler);
        let packet_codec = self.namespace.config.packet_codec;
        self.event_handlers.insert(
            event.into(),
            Arc::new(move |connection, bytes: Option<&[u8]>| {
                let handler = handler.clone();
                Box::pin(async move {
                    let bytes = match bytes {
                        Some(bytes) => bytes,
                        None => packet_codec.empty_data_encoded(),
                    };

                    let data = packet_codec.decode_data::<D>(bytes)?;
                    handler(connection, &data).await
                })
            }),
        );

        Ok(())
    }

    pub async fn on_disconnect<H, Fut>(&self, handler: H)
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.on_disconnect_handler
            .lock()
            .await
            .replace(Box::new(move |connection| Box::pin(handler(connection))));
    }

    #[inline]
    pub fn server(&self) -> WsIoServer {
        self.namespace.server()
    }

    #[inline]
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
