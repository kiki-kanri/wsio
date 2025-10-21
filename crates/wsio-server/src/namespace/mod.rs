use std::sync::Arc;

use anyhow::Result;
use bson::oid::ObjectId;
use dashmap::DashMap;
use futures_util::{
    SinkExt,
    StreamExt,
};
use http::HeaderMap;
use hyper::upgrade::OnUpgrade;
use hyper_util::rt::TokioIo;
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
    core::packet::WsIoPacket,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerNamespace {
    pub(crate) config: WsIoServerNamespaceConfig,
    connections: DashMap<String, Arc<WsIoServerConnection>>,
    connection_tasks: DashMap<String, JoinHandle<()>>,
    pub(crate) runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            connection_tasks: DashMap::new(),
            runtime,
        })
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

    pub(crate) fn handle_on_upgrade_request(
        self: &Arc<Self>,
        headers: HeaderMap,
        namespace: Arc<WsIoServerNamespace>,
        on_upgrade: OnUpgrade,
    ) {
        let connection_sid = ObjectId::new().to_string();
        let runtime = self.clone();
        self.connection_tasks.insert(
            connection_sid.clone(),
            spawn(async move {
                if let Ok(upgraded) = on_upgrade.await {
                    let ws_stream = WebSocketStream::from_raw_socket(
                        TokioIo::new(upgraded),
                        Role::Server,
                        Some(namespace.config.websocket_config),
                    )
                    .await;

                    let (connection, mut message_rx) =
                        WsIoServerConnection::new(headers, namespace, connection_sid.clone());

                    let (mut ws_stream_writer, mut ws_stream_reader) = ws_stream.split();
                    let connection_clone = connection.clone();
                    let read_ws_stream_task = spawn(async move {
                        while let Some(message) = ws_stream_reader.next().await {
                            if match message {
                                Ok(Message::Binary(bytes)) => connection_clone.handle_incoming_packet(&bytes).await,
                                Ok(Message::Close(_)) => break,
                                Ok(Message::Text(text)) => {
                                    connection_clone.handle_incoming_packet(text.as_bytes()).await
                                }
                                Err(_) => break,
                                _ => Ok(()),
                            }
                            .is_err()
                            {
                                break;
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
                                let _ = ws_stream_writer.close().await;
                                break;
                            }
                        }
                    });

                    match connection.init().await {
                        Ok(_) => {
                            select! {
                                _ = read_ws_stream_task => {},
                                _ = write_ws_stream_task => {},
                            }
                        }
                        Err(_) => {
                            connection.close().await;
                            read_ws_stream_task.abort();
                            let _ = join!(read_ws_stream_task, write_ws_stream_task);
                        }
                    }

                    connection.cleanup().await;
                }

                runtime.connection_tasks.remove(&connection_sid);
            }),
        );
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.sid().into(), connection.clone());
        self.runtime.insert_connection(connection);
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
}
