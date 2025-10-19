use crate::core::packet::codecs::WsIoPacketCodec;

#[derive(Debug)]
pub(crate) struct WsIoClientConfig {
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(crate) request_path: String,
}
