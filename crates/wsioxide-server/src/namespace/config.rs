use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
};

pub(super) struct WsIoServerNamespaceConfig {
    pub(super) auth_handler: Option<()>,
    pub(super) on_connect_handler:
        Arc<Box<dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>>,

    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
