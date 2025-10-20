use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::{
    Result,
    bail,
};
use futures::{
    SinkExt,
    StreamExt,
};
use tokio::{
    select,
    spawn,
    sync::{
        Mutex,
        RwLock,
    },
    task::JoinHandle,
    time::sleep,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
};

use crate::{
    config::WsIoClientConfig,
    connection::WsIoClientConnection,
    core::packet::WsIoPacket,
};

enum WsIoClientRuntimeStatus {
    Running,
    Starting,
    Stopped,
    Stopping,
}

pub(crate) struct WsIoClientRuntime {
    pub(crate) config: WsIoClientConfig,
    connection: RwLock<Option<Arc<WsIoClientConnection>>>,
    connection_loop_task: Mutex<Option<JoinHandle<()>>>,
    status: RwLock<WsIoClientRuntimeStatus>,
}

impl WsIoClientRuntime {
    pub(crate) fn new(config: WsIoClientConfig) -> Self {
        Self {
            config,
            connection: RwLock::new(None),
            connection_loop_task: Mutex::new(None),
            status: RwLock::new(WsIoClientRuntimeStatus::Stopped),
        }
    }

    // Protected methods
    pub(crate) async fn connect(self: &Arc<Self>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoClientRuntimeStatus::Running | WsIoClientRuntimeStatus::Starting => return Ok(()),
                WsIoClientRuntimeStatus::Stopped => *status = WsIoClientRuntimeStatus::Starting,
                WsIoClientRuntimeStatus::Stopping => bail!("Client is stopping"),
            }
        }

        let runtime = self.clone();
        *self.connection_loop_task.lock().await = Some(spawn(async move { runtime.run_connection_loop().await }));
        Ok(())
    }

    pub(crate) async fn disconnect(self: &Arc<Self>) -> Result<()> {
        {
            let mut status = self.status.write().await;
            match *status {
                WsIoClientRuntimeStatus::Stopped | WsIoClientRuntimeStatus::Stopping => return Ok(()),
                WsIoClientRuntimeStatus::Running => *status = WsIoClientRuntimeStatus::Stopping,
                WsIoClientRuntimeStatus::Starting => bail!("Client is starting"),
            }
        }

        if let Some(connection) = &*self.connection.read().await {
            connection.close().await;
        }

        if let Some(connection_loop_task) = self.connection_loop_task.lock().await.take() {
            let _ = connection_loop_task.await;
        }

        *self.status.write().await = WsIoClientRuntimeStatus::Stopped;
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
        let (ws_stream, _) = connect_async(self.config.connect_url.as_str()).await?;

        let (connection, mut message_rx) = WsIoClientConnection::new(self.clone());
        let connection = Arc::new(connection);

        let (mut ws_stream_writer, mut ws_stream_reader) = ws_stream.split();

        let connection_clone = connection.clone();
        let read_ws_stream_task = spawn(async move {
            while let Some(message) = ws_stream_reader.next().await {
                match message {
                    Ok(Message::Binary(bytes)) => connection_clone.handle_incoming_packet(&bytes).await,
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Text(text)) => connection_clone.handle_incoming_packet(text.as_bytes()).await,
                    Err(_) => break,
                    _ => {}
                }
            }
        });

        let write_ws_stream_task = spawn(async move {
            while let Some(message) = message_rx.recv().await {
                let is_close = matches!(message, Message::Close(_));
                if ws_stream_writer.send(message).await.is_err() {
                    break;
                }

                if is_close {
                    let _ = ws_stream_writer.flush().await;
                    break;
                }
            }
        });

        connection.init().await;
        *self.connection.write().await = Some(connection.clone());
        select! {
            _ = read_ws_stream_task => {},
            _ = write_ws_stream_task => {},
        }

        *self.connection.write().await = None;
        connection.cleanup().await;
        Ok(())
    }

    pub(crate) async fn run_connection_loop(self: &Arc<Self>) {
        loop {
            {
                let mut status = self.status.write().await;
                match *status {
                    WsIoClientRuntimeStatus::Running => {}
                    WsIoClientRuntimeStatus::Starting => *status = WsIoClientRuntimeStatus::Running,
                    _ => break,
                }
            }

            let _ = self.run_connection().await;
            if matches!(*self.status.read().await, WsIoClientRuntimeStatus::Running) {
                sleep(Duration::from_secs(1)).await;
            } else {
                break;
            }
        }
    }
}
