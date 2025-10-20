use std::{
    pin::Pin,
    sync::Arc,
};

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
};

#[derive(Debug)]
enum WsIoServerConnectionStatus {
    Activating,
    Authenticating,
    AwaitingAuth,
    Closed,
    Closing,
    Created,
    Ready,
}

type EventHandler = Box<
    dyn for<'a> Fn(Arc<WsIoServerConnection>, Option<&'a [u8]>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

type OnCloseHandler = Box<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub struct WsIoServerConnection {
    auth_timeout_task: Mutex<Option<JoinHandle<()>>>,
    event_handlers: DashMap<String, EventHandler>,
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    on_close_handler: Mutex<Option<OnCloseHandler>>,
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
                on_close_handler: Mutex::new(None),
                sid: ObjectId::new().to_string(),
                status: RwLock::new(WsIoServerConnectionStatus::Created),
                tx,
            },
            rx,
        )
    }

    // Private methods
    async fn abort_auth_timeout_task(&self) {
        if let Some(auth_timeout_task) = self.auth_timeout_task.lock().await.take() {
            auth_timeout_task.abort();
        }
    }

    async fn activate(self: &Arc<Self>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoServerConnectionStatus::Authenticating | WsIoServerConnectionStatus::Created => {
                    *status = WsIoServerConnectionStatus::Activating
                }
                _ => bail!("Cannot activate connection in invalid status: {:#?}", *status),
            }
        }

        if let Some(middleware) = &self.namespace.config.middleware {
            middleware(self.clone()).await?;
        }

        if let Some(on_connect_handler) = &self.namespace.config.on_connect_handler {
            on_connect_handler(self.clone()).await?;
        }

        self.namespace.insert_connection(self.clone());
        self.send_packet(&WsIoPacket {
            data: None,
            key: None,
            r#type: WsIoPacketType::Ready,
        })?;

        *self.status.write().await = WsIoServerConnectionStatus::Ready;
        if let Some(on_ready_handler) = &self.namespace.config.on_ready_handler {
            on_ready_handler(self.clone()).await?;
        }

        Ok(())
    }

    async fn handle_auth_packet(self: &Arc<Self>, packet_data: Option<&[u8]>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoServerConnectionStatus::AwaitingAuth => *status = WsIoServerConnectionStatus::Authenticating,
                _ => bail!("Received auth packet in invalid status: {:#?}", *status),
            }
        }

        if let Some(auth_handler) = &self.namespace.config.auth_handler {
            (auth_handler)(self.clone(), packet_data).await?;
            self.abort_auth_timeout_task().await;
            let status = self.status.read().await;
            if !matches!(*status, WsIoServerConnectionStatus::Authenticating) {
                bail!("Auth packet processed while connection status was {:#?}", *status);
            }

            self.activate().await?;
            Ok(())
        } else {
            bail!("Auth packet received but no auth handler is configured");
        }
    }

    #[inline]
    fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        Ok(self.tx.send(self.namespace.encode_packet_to_message(packet)?)?)
    }

    // Protected methods
    pub(crate) async fn cleanup(self: &Arc<Self>) {
        *self.status.write().await = WsIoServerConnectionStatus::Closing;
        self.abort_auth_timeout_task().await;
        self.namespace.remove_connection(&self.sid);
        if let Some(on_close_handler) = self.on_close_handler.lock().await.take() {
            let _ = on_close_handler(self.clone()).await;
        }

        *self.status.write().await = WsIoServerConnectionStatus::Closed;
    }

    pub(crate) async fn close(&self) {
        {
            let mut status = self.status.write().await;
            if matches!(
                *status,
                WsIoServerConnectionStatus::Closed | WsIoServerConnectionStatus::Closing
            ) {
                return;
            }

            *status = WsIoServerConnectionStatus::Closing;
        }

        let _ = self.tx.send(Message::Close(None));
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) {
        let packet = match self.namespace.config.packet_codec.decode(bytes) {
            Ok(packet) => packet,
            Err(_) => return,
        };

        if match packet.r#type {
            WsIoPacketType::Auth => self.handle_auth_packet(packet.data.as_deref()).await,
            WsIoPacketType::Event => Ok(()),
            _ => Ok(()),
        }
        .is_err()
        {
            self.close().await;
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        let require_auth = self.namespace.config.auth_handler.is_some();
        let packet = WsIoPacket {
            data: Some(self.namespace.config.packet_codec.encode_data(&require_auth)?),
            key: None,
            r#type: WsIoPacketType::Init,
        };

        if require_auth {
            *self.status.write().await = WsIoServerConnectionStatus::AwaitingAuth;
            let connection = self.clone();
            *self.auth_timeout_task.lock().await = Some(spawn(async move {
                sleep(connection.namespace.config.auth_timeout).await;
                if matches!(
                    *connection.status.read().await,
                    WsIoServerConnectionStatus::AwaitingAuth
                ) {
                    connection.close().await;
                }
            }));

            self.send_packet(&packet)?;
        } else {
            self.send_packet(&packet)?;
            self.activate().await?;
        }

        Ok(())
    }

    // Public methods
    pub async fn disconnect(&self) {
        let _ = self.send_packet(&WsIoPacket {
            data: None,
            key: None,
            r#type: WsIoPacketType::Disconnect,
        });

        // TODO: Should we wait for the disconnect packet to be sent or use spawn?
        let _ = self.close().await;
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn namespace(&self) -> Arc<WsIoServerNamespace> {
        self.namespace.clone()
    }

    #[inline]
    pub fn off(&self, event: impl AsRef<str>) {
        self.event_handlers.remove(event.as_ref());
    }

    #[inline]
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
            Box::new(move |connection, bytes: Option<&[u8]>| {
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

    pub async fn on_close<H, Fut>(&self, handler: H)
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        *self.on_close_handler.lock().await = Some(Box::new(move |connection| Box::pin(handler(connection))));
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
