use std::sync::Arc;

use anyhow::{
    Result,
    anyhow,
    bail,
};
use tokio::{
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

use crate::{
    core::packet::{
        WsIoPacket,
        WsIoPacketType,
    },
    runtime::WsIoClientRuntime,
};

#[derive(Debug)]
pub(crate) enum WsIoClientConnectionStatus {
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
    init_timeout_task: Mutex<Option<JoinHandle<()>>>,
    message_tx: Sender<Message>,
    ready_timeout_task: Mutex<Option<JoinHandle<()>>>,
    runtime: Arc<WsIoClientRuntime>,
    status: RwLock<WsIoClientConnectionStatus>,
}

impl WsIoClientConnection {
    pub(crate) fn new(runtime: Arc<WsIoClientRuntime>) -> (Self, Receiver<Message>) {
        // TODO: use config set buf size
        let (message_tx, message_rx) = channel(512);
        (
            Self {
                init_timeout_task: Mutex::new(None),
                message_tx,
                ready_timeout_task: Mutex::new(None),
                runtime,
                status: RwLock::new(WsIoClientConnectionStatus::Created),
            },
            message_rx,
        )
    }

    // Private methods
    async fn handle_init_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoClientConnectionStatus::AwaitingInit => *status = WsIoClientConnectionStatus::Initiating,
                _ => bail!("Received init packet in invalid status: {:#?}", *status),
            }
        }

        if let Some(init_timeout_task) = self.init_timeout_task.lock().await.take() {
            init_timeout_task.abort();
        }

        let requires_auth = self.runtime.config.packet_codec.decode_data::<bool>(bytes)?;
        *self.status.write().await = WsIoClientConnectionStatus::AwaitingReady;
        let connection = self.clone();
        *self.ready_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.runtime.config.ready_timeout).await;
            if matches!(
                *connection.status.read().await,
                WsIoClientConnectionStatus::AwaitingReady
            ) {
                connection.close().await;
            }
        }));

        if requires_auth {
            if let Some(auth_handler) = &self.runtime.config.auth_handler {
                self.send_packet(&WsIoPacket {
                    key: None,
                    data: auth_handler(self.clone()).await?,
                    r#type: WsIoPacketType::Auth,
                })
                .await?;
            } else {
                bail!("Auth required but no auth handler is configured");
            }
        }

        Ok(())
    }

    async fn handle_ready_packet(self: &Arc<Self>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoClientConnectionStatus::AwaitingReady => *status = WsIoClientConnectionStatus::Readying,
                _ => bail!("Received ready packet in invalid status: {:#?}", *status),
            }
        }

        if let Some(ready_timeout_task) = self.ready_timeout_task.lock().await.take() {
            ready_timeout_task.abort();
        }

        *self.status.write().await = WsIoClientConnectionStatus::Ready;
        if let Some(on_connection_ready_handler) = &self.runtime.config.on_connection_ready_handler {
            on_connection_ready_handler(self.clone()).await?;
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
        *self.status.write().await = WsIoClientConnectionStatus::Closing;
        if let Some(init_timeout_task) = self.init_timeout_task.lock().await.take() {
            init_timeout_task.abort();
        }

        if let Some(ready_timeout_task) = self.ready_timeout_task.lock().await.take() {
            ready_timeout_task.abort();
        }

        if let Some(on_connection_close_handler) = &self.runtime.config.on_connection_close_handler {
            let _ = on_connection_close_handler(self.clone()).await;
        }

        *self.status.write().await = WsIoClientConnectionStatus::Closed;
    }

    pub(crate) async fn close(&self) {
        {
            let mut status = self.status.write().await;
            if matches!(
                *status,
                WsIoClientConnectionStatus::Closed | WsIoClientConnectionStatus::Closing
            ) {
                return;
            }

            *status = WsIoClientConnectionStatus::Closing;
        }

        let _ = self.message_tx.send(Message::Close(None)).await;
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) {
        let packet = match self.runtime.config.packet_codec.decode(bytes) {
            Ok(packet) => packet,
            Err(_) => return,
        };

        if match packet.r#type {
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
        .is_err()
        {
            self.close().await;
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) {
        *self.status.write().await = WsIoClientConnectionStatus::AwaitingInit;
        let connection = self.clone();
        *self.init_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.runtime.config.init_timeout).await;
            if matches!(
                *connection.status.read().await,
                WsIoClientConnectionStatus::AwaitingInit
            ) {
                connection.close().await;
            }
        }));
    }
}
