use crate::packet::codecs::WsIoPacketCodec;

#[derive(Clone, Debug)]
pub(super) struct WsIoNamespaceConfig {
    pub(super) path: String,
    pub(super) packet_codec: WsIoPacketCodec,
}
