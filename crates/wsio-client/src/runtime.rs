use std::sync::Arc;

use anyhow::Result;
use arc_swap::ArcSwapOption;
use futures_util::{
    SinkExt,
    StreamExt,
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
        Notify,
    },
    task::JoinHandle,
    time::sleep,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::Message,
};

use crate::{
    config::WsIoClientConfig,
    connection::WsIoClientConnection,
    core::{
        atomic::status::AtomicStatus,
        packet::WsIoPacket,
    },
};

#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
enum RuntimeStatus {
    Running,
    Stopped,
    Stopping,
}

pub(crate) struct WsIoClientRuntime {
    break_run_connection_loop_notify: ArcSwapOption<Notify>,
    pub(crate) config: WsIoClientConfig,
    connection: ArcSwapOption<WsIoClientConnection>,
    connection_loop_task: Mutex<Option<JoinHandle<()>>>,
    operate_lock: Mutex<()>,
    status: AtomicStatus<RuntimeStatus>,
}

impl WsIoClientRuntime {
    pub(crate) fn new(config: WsIoClientConfig) -> Arc<Self> {
        Arc::new(Self {
            break_run_connection_loop_notify: ArcSwapOption::new(None),
            config,
            connection: ArcSwapOption::new(None),
            connection_loop_task: Mutex::new(None),
            operate_lock: Mutex::new(()),
            status: AtomicStatus::new(RuntimeStatus::Stopped),
        })
    }

    // Protected methods
    pub(crate) async fn connect(self: &Arc<Self>) -> Result<()> {
        // Lock to prevent concurrent operation
        let _lock = self.operate_lock.lock().await;

        match self.status.get() {
            RuntimeStatus::Running => return Ok(()),
            RuntimeStatus::Stopped => self.status.store(RuntimeStatus::Running),
            _ => unreachable!(),
        }

        let break_notify = Arc::new(Notify::new());
        self.break_run_connection_loop_notify.store(Some(break_notify.clone()));
        let runtime = self.clone();
        *self.connection_loop_task.lock().await =
            Some(spawn(async move { runtime.run_connection_loop(break_notify).await }));

        Ok(())
    }

    pub(crate) async fn disconnect(self: &Arc<Self>) -> Result<()> {
        // Lock to prevent concurrent operation
        let _lock = self.operate_lock.lock().await;

        match self.status.get() {
            RuntimeStatus::Stopped | RuntimeStatus::Stopping => return Ok(()),
            RuntimeStatus::Running => self.status.store(RuntimeStatus::Stopping),
        }

        if let Some(break_run_connection_loop_notify) = self.break_run_connection_loop_notify.load_full() {
            break_run_connection_loop_notify.notify_one();
        }

        if let Some(connection) = self.connection.load_full() {
            connection.close();
        }

        if let Some(connection_loop_task) = self.connection_loop_task.lock().await.take() {
            let _ = connection_loop_task.await;
        }

        self.status.store(RuntimeStatus::Stopped);
        Ok(())
    }

    #[inline]
    pub(crate) fn encode_packet_to_message(&self, packet: &WsIoPacket) -> Result<Message> {
        let bytes = self.config.packet_codec.encode(packet)?;
        Ok(match self.config.packet_codec.is_text() {
            true => Message::Text(unsafe { String::from_utf8_unchecked(bytes).into() }),
            false => Message::Binary(bytes.into()),
        })
    }

    pub(crate) async fn run_connection(self: &Arc<Self>) -> Result<()> {
        let (ws_stream, _) = connect_async_with_config(
            self.config.connect_url.as_str(),
            Some(self.config.websocket_config),
            false,
        )
        .await?;

        let (connection, mut message_rx) = WsIoClientConnection::new(self.clone());
        connection.init().await;

        let (mut ws_stream_writer, mut ws_stream_reader) = ws_stream.split();
        let connection_clone = connection.clone();
        let mut read_ws_stream_task = spawn(async move {
            while let Some(message) = ws_stream_reader.next().await {
                if match message {
                    Ok(Message::Binary(bytes)) => connection_clone.handle_incoming_packet(&bytes).await,
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Text(text)) => connection_clone.handle_incoming_packet(text.as_bytes()).await,
                    Err(_) => break,
                    _ => Ok(()),
                }
                .is_err()
                {
                    break;
                }
            }
        });

        let mut write_ws_stream_task = spawn(async move {
            while let Some(message) = message_rx.recv().await {
                let is_close = matches!(message, Message::Close(_));
                if ws_stream_writer.send(message).await.is_err() {
                    break;
                }

                if is_close {
                    let _ = ws_stream_writer.close().await;
                    break;
                }
            }
        });

        self.connection.store(Some(connection.clone()));
        select! {
            _ = &mut read_ws_stream_task => {
                write_ws_stream_task.abort();
            },
            _ = &mut write_ws_stream_task => {
                read_ws_stream_task.abort();
            },
        }

        self.connection.store(None);
        connection.cleanup().await;
        Ok(())
    }

    pub(crate) async fn run_connection_loop(self: &Arc<Self>, break_notify: Arc<Notify>) {
        loop {
            if !self.status.is(RuntimeStatus::Running) {
                break;
            }

            let _ = self.run_connection().await;
            if self.status.is(RuntimeStatus::Running) {
                select! {
                    _ = break_notify.notified() => {
                        break;
                    },
                    _ = sleep(self.config.reconnection_delay) => {},
                }
            } else {
                break;
            }
        }
    }
}
