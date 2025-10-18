use std::{
    sync::Arc,
    time::Duration,
};

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
            config: WsIoServerConfig {
                auth_timeout: Duration::from_secs(5),
                default_packet_codec: WsIoPacketCodec::SerdeJson,
                request_path: "/ws.io".into(),
            },
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
