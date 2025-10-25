use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
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

use crate::{
    core::{
        atomic::status::AtomicStatus,
        channel_capacity_from_websocket_config,
        packet::{
            WsIoPacket,
            WsIoPacketType,
        },
        utils::task::abort_locked_task,
    },
    runtime::WsIoClientRuntime,
};

#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
enum ConnectionStatus {
    AwaitingInit,
    AwaitingReady,
    Closed,
    Closing,
    Created,
    Initiating,
    Ready,
    Readying,
}

pub struct WsIoClientConnection {
    cancel_token: CancellationToken,
    init_timeout_task: Mutex<Option<JoinHandle<()>>>,
    message_tx: Sender<Message>,
    ready_timeout_task: Mutex<Option<JoinHandle<()>>>,
    runtime: Arc<WsIoClientRuntime>,
    status: AtomicStatus<ConnectionStatus>,
}

impl WsIoClientConnection {
    #[inline]
    pub(crate) fn new(runtime: Arc<WsIoClientRuntime>) -> (Arc<Self>, Receiver<Message>) {
        let channel_capacity = channel_capacity_from_websocket_config(runtime.config.websocket_config);
        let (message_tx, message_rx) = channel(channel_capacity);
        (
            Arc::new(Self {
                cancel_token: CancellationToken::new(),
                init_timeout_task: Mutex::new(None),
                message_tx,
                ready_timeout_task: Mutex::new(None),
                runtime,
                status: AtomicStatus::new(ConnectionStatus::Created),
            }),
            message_rx,
        )
    }

    // Private methods
    async fn handle_disconnect_packet(&self) -> Result<()> {
        let runtime = self.runtime.clone();
        spawn(async move { runtime.disconnect().await });
        Ok(())
    }

    async fn handle_init_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        // Verify current state; only valid from AwaitingInit → Initiating
        let status = self.status.get();
        match status {
            ConnectionStatus::AwaitingInit => self.status.try_transition(status, ConnectionStatus::Initiating)?,
            _ => bail!("Received init packet in invalid status: {:#?}", status),
        }

        // Abort init-timeout task if still active
        abort_locked_task(&self.init_timeout_task).await;

        // Decode init packet to determine if authentication is required
        let requires_auth = self.runtime.config.packet_codec.decode_data::<bool>(bytes)?;

        // Transition state to AwaitingReady
        self.status
            .try_transition(ConnectionStatus::Initiating, ConnectionStatus::AwaitingReady)?;

        // Spawn ready-timeout watchdog to close connection if Ready is not received in time
        let connection = self.clone();
        *self.ready_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.runtime.config.ready_packet_timeout).await;
            if connection.status.is(ConnectionStatus::AwaitingReady) {
                connection.close();
            }
        }));

        // If authentication is required
        if requires_auth {
            // Ensure auth handler is configured
            if let Some(auth_handler) = &self.runtime.config.auth_handler {
                // Execute auth handler with timeout protection
                let auth_data = timeout(
                    self.runtime.config.auth_handler_timeout,
                    auth_handler(self.clone(), &self.runtime.config.packet_codec),
                )
                .await??;

                // Send Auth packet only if still in AwaitingReady state
                if self.status.is(ConnectionStatus::AwaitingReady) {
                    self.send_packet(&WsIoPacket::new(WsIoPacketType::Auth, None, Some(auth_data)))
                        .await?;
                }
            } else {
                bail!("Auth required but no auth handler is configured");
            }
        }

        Ok(())
    }

    async fn handle_ready_packet(self: &Arc<Self>) -> Result<()> {
        // Verify current state; only valid from AwaitingReady → Ready
        let status = self.status.get();
        match status {
            ConnectionStatus::AwaitingReady => self.status.try_transition(status, ConnectionStatus::Ready)?,
            _ => bail!("Received ready packet in invalid status: {:#?}", status),
        }

        // Abort ready-timeout task if still active
        abort_locked_task(&self.ready_timeout_task).await;

        // Wake event message flush task
        self.runtime.event_message_flush_notify.notify_waiters();

        // Invoke on_connection_ready handler if configured
        if let Some(on_connection_ready_handler) = self.runtime.config.on_connection_ready_handler.clone() {
            // Run handler asynchronously in a detached task
            let connection = self.clone();
            self.spawn_task(async move { on_connection_ready_handler(connection).await });
        }

        Ok(())
    }

    async fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        Ok(self
            .message_tx
            .send(self.runtime.encode_packet_to_message(packet)?)
            .await?)
    }

    // Protected methods
    pub(crate) async fn cleanup(self: &Arc<Self>) {
        // Set connection state to Closing
        self.status.store(ConnectionStatus::Closing);

        // Abort timeout tasks if still active
        abort_locked_task(&self.init_timeout_task).await;
        abort_locked_task(&self.ready_timeout_task).await;

        // Cancel all ongoing operations via cancel token
        self.cancel_token.cancel();

        // Invoke on_connection_close handler with timeout protection if configured
        if let Some(on_connection_close_handler) = &self.runtime.config.on_connection_close_handler {
            let _ = timeout(
                self.runtime.config.on_connection_close_handler_timeout,
                on_connection_close_handler(self.clone()),
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
        let packet = self.runtime.config.packet_codec.decode(bytes)?;
        match packet.r#type {
            WsIoPacketType::Disconnect => self.handle_disconnect_packet().await,
            WsIoPacketType::Init => {
                if let Some(packet_data) = packet.data.as_deref() {
                    self.handle_init_packet(packet_data).await
                } else {
                    bail!("Init packet missing data");
                }
            }
            WsIoPacketType::Ready => self.handle_ready_packet().await,
            _ => Ok(()),
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) {
        self.status.store(ConnectionStatus::AwaitingInit);
        let connection = self.clone();
        *self.init_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.runtime.config.init_packet_timeout).await;
            if connection.status.is(ConnectionStatus::AwaitingInit) {
                connection.close();
            }
        }));
    }

    pub(crate) async fn send_event_message(&self, message: Message) -> Result<()> {
        let status = self.status.get();
        if status != ConnectionStatus::Ready {
            bail!("Cannot send event message in invalid status: {:#?}", status);
        }

        Ok(self.message_tx.send(message).await?)
    }

    // Public methods

    #[inline]
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
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
