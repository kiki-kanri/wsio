use std::sync::Arc;

use anyhow::Result;

use super::{
    WsIoServerNamespace,
    config::WsIoServerNamespaceConfig,
};
use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerNamespaceBuilder {
    config: WsIoServerNamespaceConfig,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespaceBuilder {
    pub(crate) fn new<H, Fut>(path: &str, on_connect_handler: H, runtime: Arc<WsIoServerRuntime>) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let config = WsIoServerNamespaceConfig {
            auth_handler: None,
            on_connect_handler: Arc::new(Box::new(move |connection| Box::pin(on_connect_handler(connection)))),
            // on_connect_handler: Arc::new(Box::new(move |connection| Box::pin(on_connect_handler(connection)))),
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
