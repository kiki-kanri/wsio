use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::{
    Result,
    bail,
};
use bson::oid::ObjectId;
use http::HeaderMap;
use serde::Serialize;
use tokio::{
    select,
    spawn,
    sync::{
        Mutex,
        RwLock,
        mpsc::{
            Receiver,
            Sender,
            channel,
        },
    },
    task::JoinHandle,
    time::sleep,
};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "connection-extensions")]
mod extensions;

#[cfg(feature = "connection-extensions")]
use self::extensions::WsIoServerConnectionExtensions;
use crate::{
    WsIoServer,
    core::packet::{
        WsIoPacket,
        WsIoPacketType,
    },
    namespace::WsIoServerNamespace,
};

#[derive(Debug)]
enum ConnectionStatus {
    Activating,
    Authenticating,
    AwaitingAuth,
    Closed,
    Closing,
    Created,
    Ready,
}

type OnCloseHandler = Box<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub struct WsIoServerConnection {
    auth_timeout_task: Mutex<Option<JoinHandle<()>>>,
    cancel_token: CancellationToken,
    #[cfg(feature = "connection-extensions")]
    extensions: WsIoServerConnectionExtensions,
    headers: HeaderMap,
    message_tx: Sender<Message>,
    namespace: Arc<WsIoServerNamespace>,
    on_close_handler: Mutex<Option<OnCloseHandler>>,
    sid: String,
    status: RwLock<ConnectionStatus>,
}

impl WsIoServerConnection {
    pub(crate) fn new(headers: HeaderMap, namespace: Arc<WsIoServerNamespace>) -> (Self, Receiver<Message>) {
        // TODO: use config set buf size
        let (message_tx, message_rx) = channel(512);
        (
            Self {
                auth_timeout_task: Mutex::new(None),
                cancel_token: CancellationToken::new(),
                #[cfg(feature = "connection-extensions")]
                extensions: WsIoServerConnectionExtensions::new(),
                headers,
                message_tx,
                namespace,
                on_close_handler: Mutex::new(None),
                sid: ObjectId::new().to_string(),
                status: RwLock::new(ConnectionStatus::Created),
            },
            message_rx,
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
                ConnectionStatus::Authenticating | ConnectionStatus::Created => *status = ConnectionStatus::Activating,
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
        })
        .await?;

        *self.status.write().await = ConnectionStatus::Ready;
        if let Some(on_ready_handler) = &self.namespace.config.on_ready_handler {
            on_ready_handler(self.clone()).await?;
        }

        Ok(())
    }

    async fn handle_auth_packet(self: &Arc<Self>, packet_data: Option<&[u8]>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                ConnectionStatus::AwaitingAuth => *status = ConnectionStatus::Authenticating,
                _ => bail!("Received auth packet in invalid status: {:#?}", *status),
            }
        }

        if let Some(auth_handler) = &self.namespace.config.auth_handler {
            (auth_handler)(self.clone(), packet_data).await?;
            self.abort_auth_timeout_task().await;
            self.activate().await
        } else {
            bail!("Auth packet received but no auth handler is configured");
        }
    }

    async fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        Ok(self
            .message_tx
            .send(self.namespace.encode_packet_to_message(packet)?)
            .await?)
    }

    // Protected methods
    pub(crate) async fn cleanup(self: &Arc<Self>) {
        *self.status.write().await = ConnectionStatus::Closing;
        self.abort_auth_timeout_task().await;
        self.namespace.remove_connection(&self.sid);
        self.cancel_token.cancel();
        if let Some(on_close_handler) = self.on_close_handler.lock().await.take() {
            let _ = on_close_handler(self.clone()).await;
        }

        *self.status.write().await = ConnectionStatus::Closed;
    }

    pub(crate) async fn close(&self) {
        {
            let mut status = self.status.write().await;
            match *status {
                ConnectionStatus::Closed | ConnectionStatus::Closing => return,
                _ => *status = ConnectionStatus::Closing,
            }
        }

        let _ = self.message_tx.send(Message::Close(None)).await;
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        let packet = self.namespace.config.packet_codec.decode(bytes)?;
        match packet.r#type {
            WsIoPacketType::Auth => self.handle_auth_packet(packet.data.as_deref()).await,
            _ => Ok(()),
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
            *self.status.write().await = ConnectionStatus::AwaitingAuth;
            let connection = self.clone();
            *self.auth_timeout_task.lock().await = Some(spawn(async move {
                sleep(connection.namespace.config.auth_timeout).await;
                if matches!(*connection.status.read().await, ConnectionStatus::AwaitingAuth) {
                    connection.close().await;
                }
            }));

            self.send_packet(&packet).await
        } else {
            self.send_packet(&packet).await?;
            self.activate().await
        }
    }

    // Public methods

    #[inline]
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }

    pub async fn disconnect(&self) {
        let _ = self
            .send_packet(&WsIoPacket {
                data: None,
                key: None,
                r#type: WsIoPacketType::Disconnect,
            })
            .await;

        self.close().await
    }

    pub async fn emit<D: Serialize>(&self, event: impl AsRef<str>, data: Option<&D>) -> Result<()> {
        self.send_packet(&WsIoPacket {
            data: data
                .map(|data| self.namespace.config.packet_codec.encode_data(data))
                .transpose()?,
            key: Some(event.as_ref().to_string()),
            r#type: WsIoPacketType::Event,
        })
        .await
    }

    #[cfg(feature = "connection-extensions")]
    #[inline]
    pub fn extensions(&self) -> &WsIoServerConnectionExtensions {
        &self.extensions
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn namespace(&self) -> Arc<WsIoServerNamespace> {
        self.namespace.clone()
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

    #[inline]
    pub fn spawn_task<F: Future<Output = Result<()>> + Send + 'static>(&self, future: F) {
        let cancel_token = self.cancel_token().clone();
        spawn(async move {
            select! {
                _ = cancel_token.cancelled() => {},
                _ = future => {},
            }
        });
    }
}
