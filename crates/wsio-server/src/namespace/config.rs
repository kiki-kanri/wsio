use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

use crate::{
    context::WsIoContext,
    packet::codecs::WsIoPacketCodec,
};

type AuthHandlerFn = Arc<dyn Fn(&WsIoContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

#[derive(Clone)]
pub(super) struct WsIoNamespaceConfig {
    pub(super) auth_handler_fn: Option<AuthHandlerFn>,
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
