use crate::packet::codecs::WsIoPacketCodec;

#[derive(Clone, Debug)]
pub(crate) struct WsIoConfig {
    pub(crate) default_packet_codec: WsIoPacketCodec,
    pub(crate) request_path: String,
}

impl Default for WsIoConfig {
    fn default() -> Self {
        Self {
            default_packet_codec: WsIoPacketCodec::SerdeJson,
            request_path: "/ws.io".into(),
        }
    }
}
