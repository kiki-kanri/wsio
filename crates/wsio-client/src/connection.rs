use std::sync::Arc;

use anyhow::{
    Result,
    anyhow,
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
        packet::{
            WsIoPacket,
            WsIoPacketType,
        },
    },
    runtime::WsIoClientRuntime,
};

#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
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
    pub(crate) fn new(runtime: Arc<WsIoClientRuntime>) -> (Arc<Self>, Receiver<Message>) {
        let channel_capacity = (runtime.config.websocket_config.max_write_buffer_size
            / runtime.config.websocket_config.write_buffer_size)
            .clamp(64, 4096);

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
        if let Some(init_timeout_task) = self.init_timeout_task.lock().await.take() {
            init_timeout_task.abort();
        }

        // Decode init packet to determine if authentication is required
        let requires_auth = self.runtime.config.packet_codec.decode_data::<bool>(bytes)?;

        // Transition connection state to AwaitingReady
        self.status
            .try_transition(ConnectionStatus::Initiating, ConnectionStatus::AwaitingReady)?;

        // Spawn ready-timeout watchdog to close connection if Ready is not received in time
        let connection = self.clone();
        *self.ready_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.runtime.config.ready_timeout).await;
            if connection.status.is(ConnectionStatus::AwaitingReady) {
                connection.close().await;
            }
        }));

        // If authentication is required
        if requires_auth {
            // Ensure auth handler is configured
            if let Some(auth_handler) = &self.runtime.config.auth_handler {
                // Execute auth handler with timeout protection
                let auth_data = timeout(self.runtime.config.auth_handler_timeout, auth_handler(self.clone())).await??;

                // Send Auth packet only if still in AwaitingReady state
                if self.status.is(ConnectionStatus::AwaitingReady) {
                    self.send_packet(&WsIoPacket {
                        data: auth_data,
                        key: None,
                        r#type: WsIoPacketType::Auth,
                    })
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
        if let Some(ready_timeout_task) = self.ready_timeout_task.lock().await.take() {
            ready_timeout_task.abort();
        }

        // Invoke on_connection_ready handler if configured
        if let Some(on_connection_ready_handler) = self.runtime.config.on_connection_ready_handler.clone() {
            let connection = self.clone();

            // Run handler asynchronously in a detached task
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

        // Abort init-timeout task if still active
        if let Some(init_timeout_task) = self.init_timeout_task.lock().await.take() {
            init_timeout_task.abort();
        }

        // Abort ready-timeout task if still active
        if let Some(ready_timeout_task) = self.ready_timeout_task.lock().await.take() {
            ready_timeout_task.abort();
        }

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

    pub(crate) async fn close(&self) {
        // Skip if connection is already Closing or Closed, otherwise set connection state to Closing
        match self.status.get() {
            ConnectionStatus::Closed | ConnectionStatus::Closing => return,
            _ => self.status.store(ConnectionStatus::Closing),
        }

        // Send websocket close frame to initiate graceful shutdown
        let _ = self.message_tx.send(Message::Close(None)).await;
    }

    // TODO: fifo?
    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        let packet = self.runtime.config.packet_codec.decode(bytes)?;
        match packet.r#type {
            WsIoPacketType::Disconnect => self.handle_disconnect_packet().await,
            WsIoPacketType::Init => {
                if let Some(packet_data) = packet.data.as_deref() {
                    self.handle_init_packet(packet_data).await
                } else {
                    Err(anyhow!("Init packet missing data"))
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
            sleep(connection.runtime.config.init_timeout).await;
            if connection.status.is(ConnectionStatus::AwaitingInit) {
                connection.close().await;
            }
        }));
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
