use std::sync::Arc;

use crate::{
    WsIoClient,
    config::WsIoClientConfig,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoClientRuntime,
};

pub struct WsIoClientBuilder {
    config: WsIoClientConfig,
}

impl WsIoClientBuilder {
    pub(crate) fn new() -> Self {
        Self {
            config: WsIoClientConfig {
                packet_codec: WsIoPacketCodec::SerdeJson,
                request_path: "/ws.io".into(),
            },
        }
    }

    // Public methods
    pub fn build(self) -> WsIoClient {
        WsIoClient(Arc::new(WsIoClientRuntime::new(self.config)))
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.request_path = request_path.as_ref().into();
        self
    }
}
