use wsioxide_core::packet::codecs::WsIoPacketCodec;

#[derive(Clone)]
pub(super) struct WsIoServerNamespaceConfig {
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
