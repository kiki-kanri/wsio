use crate::{
    core::packet::codecs::WsIoPacketCodec,
    types::handler::WsIoServerConnectionOnConnectHandler,
};

pub(super) struct WsIoServerNamespaceConfig {
    pub(super) auth_handler: Option<()>,
    pub(super) on_connect_handler: WsIoServerConnectionOnConnectHandler,
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
