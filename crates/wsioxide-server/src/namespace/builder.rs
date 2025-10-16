use std::sync::Arc;

use anyhow::Result;
use wsioxide_core::packet::codecs::WsIoPacketCodec;

use super::{
    WsIoServerNamespace,
    config::WsIoServerNamespaceConfig,
};
use crate::runtime::WsIoServerRuntime;

pub struct WsIoServerNamespaceBuilder {
    config: WsIoServerNamespaceConfig,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespaceBuilder {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoServerRuntime>) -> Self {
        let config = WsIoServerNamespaceConfig {
            auth_handler: None,
            packet_codec: runtime.config.default_packet_codec.clone(),
            path: path.into(),
        };

        Self { config, runtime }
    }

    // Public methods
    pub fn build(self) -> Result<Arc<WsIoServerNamespace>> {
        let namespace = Arc::new(WsIoServerNamespace::new(self.config, self.runtime.clone()));
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn with_packet_codec(mut self, packet_codec: WsIoPacketCodec) -> WsIoServerNamespaceBuilder {
        self.config.packet_codec = packet_codec;
        self
    }
}
