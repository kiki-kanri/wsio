use std::time::Duration;

use crate::{
    core::packet::codecs::WsIoPacketCodec,
    types::{
        handler::{
            WsIoServerConnectionOnConnectHandler,
            WsIoServerConnectionOnReadyHandler,
            WsIoServerNamespaceAuthHandler,
        },
        namespace::WsIoServerNamespaceMiddleware,
    },
};

pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<WsIoServerNamespaceAuthHandler>,
    pub(crate) auth_timeout: Duration,
    pub(crate) middleware: Option<WsIoServerNamespaceMiddleware>,
    pub(crate) on_connect_handler: Option<WsIoServerConnectionOnConnectHandler>,
    pub(crate) on_ready_handler: Option<WsIoServerConnectionOnReadyHandler>,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
