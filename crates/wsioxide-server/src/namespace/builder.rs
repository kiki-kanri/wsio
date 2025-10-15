use std::sync::Arc;

use anyhow::Result;
use wsioxide_core::packet::codecs::WsIoPacketCodec;

use super::{
    WsIoServerNamespace,
    config::WsIoServerNamespaceConfig,
};
use crate::{
    context::WsIoServerContext,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerNamespaceBuilder {
    runtime: Arc<WsIoServerRuntime>,
    config: WsIoServerNamespaceConfig,
}

impl WsIoServerNamespaceBuilder {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoServerRuntime>) -> Self {
        let config = WsIoServerNamespaceConfig {
            auth_handler_fn: None,
            packet_codec: runtime.config.default_packet_codec.clone(),
            path: path.into(),
        };

        WsIoServerNamespaceBuilder { config, runtime }
    }

    // Public methods
    pub fn build(self) -> Result<Arc<WsIoServerNamespace>> {
        let namespace = Arc::new(WsIoServerNamespace::new(self.config, self.runtime.clone()));
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn with_auth<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(&WsIoServerContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.auth_handler_fn = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }

    pub fn with_packet_codec(mut self, packet_codec: WsIoPacketCodec) -> WsIoServerNamespaceBuilder {
        self.config.packet_codec = packet_codec;
        self
    }
}
