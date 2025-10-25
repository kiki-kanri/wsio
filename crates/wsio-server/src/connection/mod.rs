use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use http::HeaderMap;
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use serde::Serialize;
use tokio::{
    select,
    spawn,
    sync::{
        Mutex,
        mpsc::{
            Receiver,
            Sender,
            channel,
        },
    },
    task::JoinHandle,
    time::{
        sleep,
        timeout,
    },
};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "connection-extensions")]
mod extensions;

#[cfg(feature = "connection-extensions")]
use self::extensions::WsIoServerConnectionExtensions;
use crate::{
    WsIoServer,
    core::{
        atomic::status::AtomicStatus,
        packet::{
            WsIoPacket,
            WsIoPacketType,
        },
        types::BoxAsyncUnaryResultHandler,
        utils::task::abort_locked_task,
    },
    namespace::WsIoServerNamespace,
};

#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
enum ConnectionStatus {
    Activating,
    Authenticating,
    AwaitingAuth,
    Closed,
    Closing,
    Created,
    Ready,
}

pub struct WsIoServerConnection {
    auth_timeout_task: Mutex<Option<JoinHandle<()>>>,
    cancel_token: CancellationToken,
    #[cfg(feature = "connection-extensions")]
    extensions: WsIoServerConnectionExtensions,
    headers: HeaderMap,
    message_tx: Sender<Message>,
    namespace: Arc<WsIoServerNamespace>,
    on_close_handler: Mutex<Option<BoxAsyncUnaryResultHandler<Self>>>,
    sid: String,
    status: AtomicStatus<ConnectionStatus>,
}

impl WsIoServerConnection {
    pub(crate) fn new(
        headers: HeaderMap,
        namespace: Arc<WsIoServerNamespace>,
        sid: String,
    ) -> (Arc<Self>, Receiver<Message>) {
        let channel_capacity = (namespace.config.websocket_config.max_write_buffer_size
            / namespace.config.websocket_config.write_buffer_size)
            .clamp(64, 4096);

        let (message_tx, message_rx) = channel(channel_capacity);
        (
            Arc::new(Self {
                auth_timeout_task: Mutex::new(None),
                cancel_token: CancellationToken::new(),
                #[cfg(feature = "connection-extensions")]
                extensions: WsIoServerConnectionExtensions::new(),
                headers,
                message_tx,
                namespace,
                on_close_handler: Mutex::new(None),
                sid,
                status: AtomicStatus::new(ConnectionStatus::Created),
            }),
            message_rx,
        )
    }

    // Private methods
    async fn activate(self: &Arc<Self>) -> Result<()> {
        // Verify current state; only valid from Authenticating or Created → Activating
        let status = self.status.get();
        match status {
            ConnectionStatus::Authenticating | ConnectionStatus::Created => {
                self.status.try_transition(status, ConnectionStatus::Activating)?
            }
            _ => bail!("Cannot activate connection in invalid status: {:#?}", status),
        }

        // Invoke middleware with timeout protection if configured
        if let Some(middleware) = &self.namespace.config.middleware {
            timeout(
                self.namespace.config.middleware_execution_timeout,
                middleware(self.clone()),
            )
            .await??;
        }

        // Invoke on_connect handler with timeout protection if configured
        if let Some(on_connect_handler) = &self.namespace.config.on_connect_handler {
            timeout(
                self.namespace.config.on_connect_handler_timeout,
                on_connect_handler(self.clone()),
            )
            .await??;
        }

        // Insert connection into namespace
        self.namespace.insert_connection(self.clone());

        // Transition state to Ready
        self.status
            .try_transition(ConnectionStatus::Activating, ConnectionStatus::Ready)?;

        // Send ready packet
        self.send_packet(&WsIoPacket {
            data: None,
            key: None,
            r#type: WsIoPacketType::Ready,
        })
        .await?;

        // Invoke on_ready handler if configured
        if let Some(on_ready_handler) = self.namespace.config.on_ready_handler.clone() {
            // Run handler asynchronously in a detached task
            let connection = self.clone();
            self.spawn_task(async move { on_ready_handler(connection).await });
        }

        Ok(())
    }

    async fn handle_auth_packet(self: &Arc<Self>, packet_data: &[u8]) -> Result<()> {
        // Verify current state; only valid from AwaitingAuth → Authenticating
        let status = self.status.get();
        match status {
            ConnectionStatus::AwaitingAuth => self.status.try_transition(status, ConnectionStatus::Authenticating)?,
            _ => bail!("Received auth packet in invalid status: {:#?}", status),
        }

        // Abort auth-timeout task if still active
        abort_locked_task(&self.auth_timeout_task).await;

        // Invoke auth handler with timeout protection if configured, otherwise raise error
        if let Some(auth_handler) = &self.namespace.config.auth_handler {
            timeout(
                self.namespace.config.auth_handler_timeout,
                auth_handler(self.clone(), packet_data, &self.namespace.config.packet_codec),
            )
            .await??;

            // Activate connection
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
        // Set connection state to Closing
        self.status.store(ConnectionStatus::Closing);

        // Abort auth-timeout task if still active
        abort_locked_task(&self.auth_timeout_task).await;

        // Remove connection from namespace
        self.namespace.remove_connection(&self.sid);

        // Cancel all ongoing operations via cancel token
        self.cancel_token.cancel();

        // Invoke on_close handler with timeout protection if configured
        if let Some(on_close_handler) = self.on_close_handler.lock().await.take() {
            let _ = timeout(
                self.namespace.config.on_close_handler_timeout,
                on_close_handler(self.clone()),
            )
            .await;
        }

        // Set connection state to Closed
        self.status.store(ConnectionStatus::Closed);
    }

    #[inline]
    pub(crate) fn close(&self) {
        // Skip if connection is already Closing or Closed, otherwise set connection state to Closing
        match self.status.get() {
            ConnectionStatus::Closed | ConnectionStatus::Closing => return,
            _ => self.status.store(ConnectionStatus::Closing),
        }

        // Send websocket close frame to initiate graceful shutdown
        let _ = self.message_tx.try_send(Message::Close(None));
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        let packet = self.namespace.config.packet_codec.decode(bytes)?;
        match packet.r#type {
            WsIoPacketType::Auth => {
                if let Some(packet_data) = packet.data.as_deref() {
                    self.handle_auth_packet(packet_data).await
                } else {
                    bail!("Auth packet missing data");
                }
            }
            _ => Ok(()),
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        // Verify current state; only valid Created
        let status = self.status.get();
        if !matches!(status, ConnectionStatus::Created) {
            bail!("Cannot init connection in invalid status: {:#?}", status);
        }

        // Determine if authentication is required
        let requires_auth = self.namespace.config.auth_handler.is_some();

        // Build Init packet to inform client whether auth is required
        let packet = WsIoPacket {
            data: Some(self.namespace.config.packet_codec.encode_data(&requires_auth)?),
            key: None,
            r#type: WsIoPacketType::Init,
        };

        // If authentication is required
        if requires_auth {
            // Transition state to AwaitingAuth
            self.status
                .try_transition(ConnectionStatus::Created, ConnectionStatus::AwaitingAuth)?;

            // Spawn auth-packet-timeout watchdog to close connection if auth not received in time
            let connection = self.clone();
            *self.auth_timeout_task.lock().await = Some(spawn(async move {
                sleep(connection.namespace.config.auth_packet_timeout).await;
                if connection.status.is(ConnectionStatus::AwaitingAuth) {
                    connection.close();
                }
            }));

            // Send Init packet to client (expecting auth response)
            self.send_packet(&packet).await
        } else {
            // Send Init packet to client (no auth required)
            self.send_packet(&packet).await?;

            // Immediately activate connection
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

        self.close()
    }

    pub async fn emit<D: Serialize>(&self, event: impl Into<String>, data: Option<&D>) -> Result<()> {
        let status = self.status.get();
        if status != ConnectionStatus::Ready {
            bail!("Cannot emit event in invalid status: {:#?}", status);
        }

        self.send_packet(&WsIoPacket {
            data: data
                .map(|data| self.namespace.config.packet_codec.encode_data(data))
                .transpose()?,
            key: Some(event.into()),
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
        let cancel_token = self.cancel_token.clone();
        spawn(async move {
            select! {
                _ = cancel_token.cancelled() => {},
                _ = future => {},
            }
        });
    }
}
