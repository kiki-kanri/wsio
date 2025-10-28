use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use futures_util::{
    SinkExt,
    StreamExt,
    future::join_all,
};
use http::HeaderMap;
use hyper::upgrade::{
    OnUpgrade,
    Upgraded,
};
use hyper_util::rt::TokioIo;
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use serde::Serialize;
use tokio::{
    join,
    select,
    spawn,
    sync::Mutex,
    task::JoinSet,
};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{
        Message,
        protocol::Role,
    },
};

pub(crate) mod builder;
mod config;

use self::config::WsIoServerNamespaceConfig;
use crate::{
    WsIoServer,
    connection::WsIoServerConnection,
    core::{
        atomic::status::AtomicStatus,
        packet::WsIoPacket,
    },
    runtime::{
        WsIoServerRuntime,
        WsIoServerRuntimeStatus,
    },
};

// Enums
#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
enum NamespaceStatus {
    Running,
    Stopped,
    Stopping,
}

// Structs
pub struct WsIoServerNamespace {
    pub(crate) config: WsIoServerNamespaceConfig,
    connections: DashMap<u64, Arc<WsIoServerConnection>>,
    connection_task_set: Mutex<JoinSet<()>>,
    runtime: Arc<WsIoServerRuntime>,
    status: AtomicStatus<NamespaceStatus>,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            connection_task_set: Mutex::new(JoinSet::new()),
            runtime,
            status: AtomicStatus::new(NamespaceStatus::Running),
        })
    }

    // Private methods
    #[inline]
    fn clone_connections(&self) -> Vec<Arc<WsIoServerConnection>> {
        self.connections.iter().map(|entry| entry.value().clone()).collect()
    }

    async fn handle_upgraded_request(self: &Arc<Self>, headers: HeaderMap, upgraded: Upgraded) -> Result<()> {
        // Create ws stream
        let mut ws_stream =
            WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Server, Some(self.config.websocket_config))
                .await;

        // Check runtime and namespace status
        if !self.runtime.status.is(WsIoServerRuntimeStatus::Running) || !self.status.is(NamespaceStatus::Running) {
            ws_stream
                .send(self.encode_packet_to_message(&WsIoPacket::new_disconnect())?)
                .await?;

            let _ = ws_stream.close(None).await;
            return Ok(());
        }

        // Create connection
        let (connection, mut message_rx) = WsIoServerConnection::new(headers, self.clone());

        // Split ws stream and spawn read and write tasks
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

        // Try to init connection
        match connection.init().await {
            Ok(_) => {
                // Wait for either read or write task to finish
                select! {
                    _ = &mut read_ws_stream_task => {
                        write_ws_stream_task.abort();
                    },
                    _ = &mut write_ws_stream_task => {
                        read_ws_stream_task.abort();
                    },
                }
            }
            Err(_) => {
                // Close connection
                read_ws_stream_task.abort();
                connection.close();
                let _ = join!(read_ws_stream_task, write_ws_stream_task);
            }
        }

        // Cleanup connection
        connection.cleanup().await;
        Ok(())
    }

    // Protected methods
    #[inline]
    pub(crate) fn encode_packet_to_message(&self, packet: &WsIoPacket) -> Result<Message> {
        let bytes = self.config.packet_codec.encode(packet)?;
        Ok(match self.config.packet_codec.is_text() {
            true => Message::Text(unsafe { String::from_utf8_unchecked(bytes).into() }),
            false => Message::Binary(bytes.into()),
        })
    }

    pub(crate) async fn handle_on_upgrade_request(self: &Arc<Self>, headers: HeaderMap, on_upgrade: OnUpgrade) {
        let namespace = self.clone();
        self.connection_task_set.lock().await.spawn(async move {
            if let Ok(upgraded) = on_upgrade.await {
                let _ = namespace.handle_upgraded_request(headers, upgraded).await;
            }
        });
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.id(), connection.clone());
        self.runtime.insert_connection(&connection);
    }

    #[inline]
    pub(crate) fn remove_connection(&self, id: u64) {
        self.connections.remove(&id);
        self.runtime.remove_connection(id);
    }

    // Public methods
    #[inline]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub async fn emit<D: Serialize>(&self, event: impl Into<String>, data: Option<&D>) -> Result<()> {
        self.status.ensure(NamespaceStatus::Running, |status| {
            format!("Cannot emit event in invalid status: {:#?}", status)
        })?;

        let message = self.encode_packet_to_message(&WsIoPacket::new_event(
            event,
            data.map(|data| self.config.packet_codec.encode_data(data))
                .transpose()?,
        ))?;

        join_all(self.clone_connections().iter().map(|connection| {
            let message = message.clone();
            async move { connection.emit_message(message).await }
        }))
        .await;

        Ok(())
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.config.path
    }

    #[inline]
    pub fn server(&self) -> WsIoServer {
        WsIoServer(self.runtime.clone())
    }

    pub async fn shutdown(&self) {
        match self.status.get() {
            NamespaceStatus::Stopped => return,
            NamespaceStatus::Running => self.status.store(NamespaceStatus::Stopping),
            _ => unreachable!(),
        }

        join_all(
            self.clone_connections()
                .iter()
                .map(|connection| connection.disconnect()),
        )
        .await;

        let mut connection_task_set = self.connection_task_set.lock().await;
        while connection_task_set.join_next().await.is_some() {}

        self.status.store(NamespaceStatus::Stopped);
    }
}
