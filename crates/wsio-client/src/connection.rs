use std::{
    sync::Arc,
    time::Duration,
};

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
    ready_timeout_task: Mutex<Option<JoinHandle<()>>>,
    runtime: Arc<WsIoClientRuntime>,
    status: RwLock<WsIoClientConnectionStatus>,
    tx: UnboundedSender<Message>,
}

impl WsIoClientConnection {
    pub(crate) fn new(runtime: Arc<WsIoClientRuntime>) -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                init_timeout_task: Mutex::new(None),
                ready_timeout_task: Mutex::new(None),
                runtime,
                status: RwLock::new(WsIoClientConnectionStatus::Created),
                tx,
            },
            rx,
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
            sleep(Duration::from_secs(3)).await;
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
                })?;
            } else {
                bail!("Auth required but no auth handler is configured");
            }
        }

        Ok(())
    }

    async fn handle_ready_packet(&self) -> Result<()> {
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
        // TODO: Handle ready
        Ok(())
    }

    #[inline]
    fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        Ok(self.tx.send(self.runtime.encode_packet_to_message(packet)?)?)
    }

    // Protected methods
    pub(crate) async fn cleanup(&self) {
        *self.status.write().await = WsIoClientConnectionStatus::Closing;
        if let Some(init_timeout_task) = self.init_timeout_task.lock().await.take() {
            init_timeout_task.abort();
        }

        if let Some(ready_timeout_task) = self.ready_timeout_task.lock().await.take() {
            ready_timeout_task.abort();
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

        let _ = self.tx.send(Message::Close(None));
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
            sleep(Duration::from_secs(3)).await;
            if matches!(
                *connection.status.read().await,
                WsIoClientConnectionStatus::AwaitingInit
            ) {
                connection.close().await;
            }
        }));
    }
}
