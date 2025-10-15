use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;
use wsioxide_core::packet::codecs::WsIoPacketCodec;

use crate::context::WsIoServerContext;

type AuthHandlerFn = Arc<dyn Fn(&WsIoServerContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

#[derive(Clone)]
pub(super) struct WsIoServerNamespaceConfig {
    pub(super) auth_handler_fn: Option<AuthHandlerFn>,
    pub(super) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
}
