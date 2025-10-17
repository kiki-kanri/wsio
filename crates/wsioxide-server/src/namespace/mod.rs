use std::sync::Arc;

use anyhow::Result;
use config::WsIoServerNamespaceConfig;
use dashmap::DashMap;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

pub(crate) mod builder;
mod config;

use crate::{
    WsIoServer,
    connection::WsIoServerConnection,
    core::packet::WsIoPacket,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerNamespace {
    config: WsIoServerNamespaceConfig,
    connections: DashMap<String, Arc<WsIoServerConnection>>,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Self {
        Self {
            config,
            connections: DashMap::new(),
            runtime,
        }
    }

    // Protected methods

    #[inline]
    pub(crate) fn cleanup_connection(&self, sid: &str) {
        self.connections.remove(sid);
        self.runtime.cleanup_connection(sid);
    }

    #[inline]
    pub(crate) fn encode_packet_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        self.config.packet_codec.encode_data(data)
    }

    pub(crate) fn encode_packet_to_message(&self, packet: &WsIoPacket) -> Result<Message> {
        let bytes = self.config.packet_codec.encode(packet)?;
        Ok(match self.config.packet_codec.is_text() {
            true => Message::Text(unsafe { String::from_utf8_unchecked(bytes).into() }),
            false => Message::Binary(bytes.into()),
        })
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.sid().into(), connection.clone());
        self.runtime.insert_connection(connection);
    }

    pub(crate) async fn on_connect(&self, connection: Arc<WsIoServerConnection>) -> Result<()> {
        (self.config.on_connect_handler)(connection).await
    }

    #[inline]
    pub(crate) fn requires_auth(&self) -> bool {
        self.config.auth_handler.is_some()
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
