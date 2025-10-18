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

pub(super) struct WsIoServerNamespaceConfig {
    pub(super) auth_handler: Option<
        Arc<
            dyn Fn(Arc<WsIoServerConnection>, Vec<u8>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub(super) auth_timeout: Duration,
    pub(super) on_connect_handler: WsIoServerConnectionOnConnectHandler,
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
