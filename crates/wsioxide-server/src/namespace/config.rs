use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;

use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
    types::handler::WsIoServerConnectionOnConnectHandler,
};

pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<
        Arc<
            dyn Fn(Arc<WsIoServerConnection>, Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub(crate) auth_timeout: Duration,
    pub(crate) on_connect_handler: WsIoServerConnectionOnConnectHandler,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
