use std::sync::Arc;

use anyhow::Result;
use bson::oid::ObjectId;
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
use tokio::{
    join,
    select,
    spawn,
    task::JoinHandle,
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
    runtime::WsIoServerRuntime,
};

#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
enum RuntimeStatus {
    Running,
    Stopped,
    Stopping,
}

pub struct WsIoServerNamespace {
    pub(crate) config: WsIoServerNamespaceConfig,
    connections: DashMap<String, Arc<WsIoServerConnection>>,
    connection_tasks: DashMap<String, JoinHandle<()>>,
    runtime: Arc<WsIoServerRuntime>,
    status: AtomicStatus<RuntimeStatus>,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            connection_tasks: DashMap::new(),
            runtime,
            status: AtomicStatus::new(RuntimeStatus::Running),
        })
    }

    // Private methods
    async fn handle_upgraded_request(self: &Arc<Self>, connection_sid: &str, headers: HeaderMap, upgraded: Upgraded) {
        // Create ws stream
        let ws_stream =
            WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Server, Some(self.config.websocket_config))
                .await;

        // Create connection
        let (connection, mut message_rx) = WsIoServerConnection::new(headers, self.clone(), connection_sid.to_string());

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

    #[inline]
    pub(crate) fn handle_on_upgrade_request(self: &Arc<Self>, headers: HeaderMap, on_upgrade: OnUpgrade) {
        let connection_sid = ObjectId::new().to_string();
        let namespace = self.clone();
        self.connection_tasks.insert(
            connection_sid.clone(),
            spawn(async move {
                if let Ok(upgraded) = on_upgrade.await {
                    namespace
                        .clone()
                        .handle_upgraded_request(&connection_sid, headers, upgraded)
                        .await;
                }

                namespace.connection_tasks.remove(&connection_sid);
            }),
        );
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.sid().into(), connection.clone());
        self.runtime.insert_connection(&connection);
    }

    #[inline]
    pub(crate) fn remove_connection(&self, sid: &str) {
        self.connections.remove(sid);
        self.runtime.remove_connection(sid);
    }

    // Public methods

    #[inline]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.config.path
    }

    #[inline]
    pub fn server(&self) -> WsIoServer {
        WsIoServer(self.runtime.clone())
    }

    // Public methods
    pub async fn shutdown(&self) {
        match self.status.get() {
            RuntimeStatus::Stopped => return,
            RuntimeStatus::Running => self.status.store(RuntimeStatus::Stopping),
            _ => unreachable!(),
        }

        join_all(self.connections.iter().map(|entry| {
            let connection = entry.value().clone();
            async move { connection.disconnect().await }
        }))
        .await;

        let connection_sids = self
            .connection_tasks
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();

        join_all(
            connection_sids
                .iter()
                .filter_map(|sid| self.connection_tasks.remove(sid).map(|(_, task)| task)),
        )
        .await;

        self.status.store(RuntimeStatus::Stopped);
    }
}
