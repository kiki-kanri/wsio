use std::sync::Arc;

use anyhow::Result;
use config::WsIoServerNamespaceConfig;
use dashmap::DashMap;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use wsioxide_core::packet::WsIoPacket;

pub(crate) mod builder;
mod config;

use crate::{
    connection::WsIoServerConnection,
    runtime::WsIoServerRuntime,
};

#[derive(Clone)]
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
    pub(crate) fn add_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.sid().into(), connection);
    }

    pub(crate) fn cleanup_connection(&self, sid: &str) {
        self.connections.remove(sid);
    }

    pub(crate) fn encode_packet_to_message<D: Serialize>(&self, packet: WsIoPacket<D>) -> Result<Message> {
        let bytes = self.config.packet_codec.encode(packet)?;
        Ok(match self.config.packet_codec.is_text() {
            true => Message::Text(unsafe { String::from_utf8_unchecked(bytes).into() }),
            false => Message::Binary(bytes.into()),
        })
    }

    #[inline]
    pub(crate) fn requires_auth(&self) -> bool {
        self.config.auth_handler.is_some()
    }

    // Public methods

    #[inline]
    pub fn path(&self) -> &str {
        &self.config.path
    }
}
