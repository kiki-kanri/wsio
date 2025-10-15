use wsioxide_core::packet::codecs::WsIoPacketCodec;

#[derive(Clone, Debug)]
pub(crate) struct WsIoServerConfig {
    pub(crate) default_packet_codec: WsIoPacketCodec,
    pub(crate) request_path: String,
}

impl Default for WsIoServerConfig {
    fn default() -> Self {
        Self {
            default_packet_codec: WsIoPacketCodec::SerdeJson,
            request_path: "/ws.io".into(),
        }
    }
}
