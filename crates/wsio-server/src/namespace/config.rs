use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;

use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
    types::handler::{
        WsIoServerConnectionOnConnectHandler,
        WsIoServerNamespaceAuthHandler,
    },
};

pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<WsIoServerNamespaceAuthHandler>,
    pub(crate) auth_timeout: Duration,
    pub(crate) middleware: Option<
        Box<
            dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub(crate) on_connect_handler: WsIoServerConnectionOnConnectHandler,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
