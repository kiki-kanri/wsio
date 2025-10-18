use std::sync::Arc;

use anyhow::Result;
use bson::oid::ObjectId;
use http::HeaderMap;
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
    core::packet::{
        WsIoPacket,
        WsIoPacketType,
    },
    namespace::WsIoServerNamespace,
    types::handler::WsIoServerConnectionOnDisconnectHandler,
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
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    on_disconnect_handler: Mutex<Option<WsIoServerConnectionOnDisconnectHandler>>,
    sid: String,
    status: RwLock<WsIoServerConnectionStatus>,
    tx: UnboundedSender<Message>,
    wait_auth_timeout_task: Mutex<Option<JoinHandle<()>>>,
}

impl WsIoServerConnection {
    pub(crate) fn new(headers: HeaderMap, namespace: Arc<WsIoServerNamespace>) -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                headers,
                namespace,
                on_disconnect_handler: Mutex::new(None),
                sid: ObjectId::new().to_string(),
                status: RwLock::new(WsIoServerConnectionStatus::Created),
                tx,
                wait_auth_timeout_task: Mutex::new(None),
            },
            rx,
        )
    }

    // Protected methods
    pub(crate) async fn activate(self: &Arc<Self>) -> Result<()> {
        *self.status.write().await = WsIoServerConnectionStatus::Activating;
        // TODO: middlewares
        self.namespace.insert_connection(self.clone());
        (self.namespace.config.on_connect_handler)(self.clone()).await?;
        *self.status.write().await = WsIoServerConnectionStatus::Ready;
        let packet = WsIoPacket {
            data: None,
            key: None,
            r#type: WsIoPacketType::Ready,
        };

        self.send_packet(&packet)
    }

    pub(crate) async fn cleanup(self: &Arc<Self>) {
        *self.status.write().await = WsIoServerConnectionStatus::Closing;
        if let Some(wait_auth_timeout_task) = self.wait_auth_timeout_task.lock().await.take() {
            wait_auth_timeout_task.abort();
        }

        self.namespace.cleanup_connection(&self.sid);
        if let Some(on_disconnect_handler) = self.on_disconnect_handler.lock().await.take() {
            let _ = on_disconnect_handler(self.clone()).await;
        }

        *self.status.write().await = WsIoServerConnectionStatus::Closed;
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        self.send(Message::Text(format!("c{}", self.namespace.config.packet_codec).into()))?;
        let require_auth = self.namespace.config.auth_handler.is_some();
        let packet = WsIoPacket {
            data: Some(self.namespace.config.packet_codec.encode_data(&require_auth)?),
            key: Some(self.sid.clone()),
            r#type: WsIoPacketType::Init,
        };

        if require_auth {
            *self.status.write().await = WsIoServerConnectionStatus::AwaitingAuth;
            let connection = self.clone();
            self.wait_auth_timeout_task.lock().await.replace(spawn(async move {
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

    pub(crate) async fn on_message(&self, _message: Message) {}

    #[inline]
    pub(crate) fn send(&self, message: Message) -> Result<()> {
        Ok(self.tx.send(message)?)
    }

    #[inline]
    pub(crate) fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        self.send(self.namespace.encode_packet_to_message(packet)?)
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
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
