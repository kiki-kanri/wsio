use std::sync::Arc;

use anyhow::Result;

use super::{
    WsIoNamespace,
    config::WsIoNamespaceConfig,
};
use crate::{
    context::WsIoContext,
    packet::codecs::WsIoPacketCodec,
    runtime::WsIoRuntime,
};

pub struct WsIoNamespaceBuilder {
    runtime: Arc<WsIoRuntime>,
    config: WsIoNamespaceConfig,
}

impl WsIoNamespaceBuilder {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoRuntime>) -> Self {
        let config = WsIoNamespaceConfig {
            auth_handler_fn: None,
            packet_codec: runtime.config.default_packet_codec.clone(),
            path: path.into(),
        };

        WsIoNamespaceBuilder { config, runtime }
    }

    // Public methods
    pub fn build(self) -> Result<Arc<WsIoNamespace>> {
        let namespace = Arc::new(WsIoNamespace::new(self.config, self.runtime.clone()));
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn with_auth<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(&WsIoContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.auth_handler_fn = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }

    pub fn with_packet_codec(mut self, packet_codec: WsIoPacketCodec) -> WsIoNamespaceBuilder {
        self.config.packet_codec = packet_codec;
        self
    }
}
