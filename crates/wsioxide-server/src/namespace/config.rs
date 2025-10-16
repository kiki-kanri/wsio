use wsioxide_core::packet::codecs::WsIoPacketCodec;

#[derive(Clone, Debug)]
pub(super) struct WsIoServerNamespaceConfig {
    pub(super) auth_handler: Option<()>,
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
