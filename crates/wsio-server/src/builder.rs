use crate::{
    WsIo,
    config::WsIoConfig,
    layer::WsIoLayer,
    packet::codecs::WsIoPacketCodec,
    runtime::WsIoRuntime,
};

pub struct WsIoBuilder {
    config: WsIoConfig,
}

impl WsIoBuilder {
    pub(crate) fn new() -> Self {
        WsIoBuilder {
            config: WsIoConfig::default(),
        }
    }

    pub fn build_layer(&self) -> (WsIoLayer, WsIo) {
        let runtime = WsIoRuntime::new(self.config.clone());
        (WsIoLayer::new(runtime.clone()), WsIo(runtime))
    }

    pub fn default_packet_codec(mut self, codec: WsIoPacketCodec) -> Self {
        self.config.default_codec = codec;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.request_path = request_path.as_ref().into();
        self
    }
}
