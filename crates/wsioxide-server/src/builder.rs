use std::sync::Arc;

use crate::{
    WsIoServer,
    config::WsIoServerConfig,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerBuilder {
    config: WsIoServerConfig,
}

impl WsIoServerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            config: WsIoServerConfig::default(),
        }
    }

    // Public methods
    pub fn build(self) -> WsIoServer {
        WsIoServer(Arc::new(WsIoServerRuntime::new(self.config)))
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.request_path = request_path.as_ref().into();
        self
    }

    pub fn with_default_packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.default_packet_codec = packet_codec;
        self
    }
}
