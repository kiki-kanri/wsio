use std::time::Duration;

use crate::{
    core::packet::codecs::WsIoPacketCodec,
    types::handler::{
        WsIoServerConnectionOnConnectHandler,
        WsIoServerNamespaceAuthHandler,
    },
};

pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<WsIoServerNamespaceAuthHandler>,
    pub(crate) auth_timeout: Duration,
    pub(crate) on_connect_handler: WsIoServerConnectionOnConnectHandler,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
