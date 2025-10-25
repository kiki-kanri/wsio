use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use arc_swap::ArcSwapOption;
use futures_util::{
    SinkExt,
    StreamExt,
};
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
        Notify,
        mpsc::{
            Receiver,
            Sender,
            channel,
        },
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
        channel_capacity_from_websocket_config,
        packet::WsIoPacket,
    },
};

#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
enum RuntimeStatus {
    Running,
    Stopped,
    Stopping,
}

pub(crate) struct WsIoClientRuntime {
    pub(crate) config: WsIoClientConfig,
    connection: ArcSwapOption<WsIoClientConnection>,
    connection_loop_task: Mutex<Option<JoinHandle<()>>>,
    pub(crate) event_message_flush_notify: Notify,
    event_message_flush_task: Mutex<Option<JoinHandle<()>>>,
    event_message_send_rx: Mutex<Receiver<Message>>,
    event_message_send_tx: Sender<Message>,
    operate_lock: Mutex<()>,
    status: AtomicStatus<RuntimeStatus>,
    wake_reconnect_wait_notify: Notify,
}

impl WsIoClientRuntime {
    pub(crate) fn new(config: WsIoClientConfig) -> Arc<Self> {
        let channel_capacity = channel_capacity_from_websocket_config(config.websocket_config);
        let (event_message_send_tx, event_message_send_rx) = channel(channel_capacity);
        Arc::new(Self {
            config,
            connection: ArcSwapOption::new(None),
            connection_loop_task: Mutex::new(None),
            event_message_flush_notify: Notify::new(),
            event_message_flush_task: Mutex::new(None),
            event_message_send_rx: Mutex::new(event_message_send_rx),
            event_message_send_tx,
            operate_lock: Mutex::new(()),
            status: AtomicStatus::new(RuntimeStatus::Stopped),
            wake_reconnect_wait_notify: Notify::new(),
        })
    }

    // Protected methods
    pub(crate) async fn connect(self: &Arc<Self>) {
        // Lock to prevent concurrent operation
        let _lock = self.operate_lock.lock().await;

        match self.status.get() {
            RuntimeStatus::Running => return,
            RuntimeStatus::Stopped => self.status.store(RuntimeStatus::Running),
            _ => unreachable!(),
        }

        // Create connection loop task
        let runtime = self.clone();
        *self.connection_loop_task.lock().await = Some(spawn(async move {
            loop {
                if !runtime.status.is(RuntimeStatus::Running) {
                    break;
                }

                let _ = runtime.run_connection().await;
                if runtime.status.is(RuntimeStatus::Running) {
                    select! {
                        _ = runtime.wake_reconnect_wait_notify.notified() => {
                            break;
                        },
                        _ = sleep(runtime.config.reconnection_delay) => {},
                    }
                } else {
                    break;
                }
            }
        }));

        // Create flush messages task
        let runtime = self.clone();
        *self.event_message_flush_task.lock().await = Some(spawn(async move {
            let mut event_message_send_rx = runtime.event_message_send_rx.lock().await;
            while let Some(message) = event_message_send_rx.recv().await {
                loop {
                    if let Some(connection) = runtime.connection.load().as_ref()
                        && connection.send_event_message(message.clone()).await.is_ok()
                    {
                        break;
                    }

                    runtime.event_message_flush_notify.notified().await;
                }
            }
        }));
    }

    pub(crate) async fn disconnect(self: &Arc<Self>) {
        // Lock to prevent concurrent operation
        let _lock = self.operate_lock.lock().await;

        match self.status.get() {
            RuntimeStatus::Stopped => return,
            RuntimeStatus::Running => self.status.store(RuntimeStatus::Stopping),
            _ => unreachable!(),
        }

        // Abort event-message-flush task if still active
        if let Some(event_message_flush_task) = self.event_message_flush_task.lock().await.take() {
            event_message_flush_task.abort();
        }

        // Drop all pending event messages in the channel
        let mut event_message_send_rx = self.event_message_send_rx.lock().await;
        while event_message_send_rx.try_recv().is_ok() {}

        // Wake reconnect loop to break out of sleep early
        self.wake_reconnect_wait_notify.notify_waiters();

        // Close connection
        if let Some(connection) = self.connection.load().as_ref() {
            connection.close();
        }

        // Await connection loop task termination
        if let Some(connection_loop_task) = self.connection_loop_task.lock().await.take() {
            let _ = connection_loop_task.await;
        }

        self.status.store(RuntimeStatus::Stopped);
    }

    pub(crate) async fn emit<D: Serialize>(&self, event: impl Into<String>, data: Option<&D>) -> Result<()> {
        let status = self.status.get();
        if status != RuntimeStatus::Running {
            bail!("Cannot emit event in invalid status: {:#?}", status);
        }

        self.event_message_send_tx
            .send(
                self.encode_packet_to_message(&WsIoPacket::new_event(
                    event,
                    data.map(|data| self.config.packet_codec.encode_data(data))
                        .transpose()?,
                ))?,
            )
            .await?;

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
}
