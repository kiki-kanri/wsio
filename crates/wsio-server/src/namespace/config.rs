use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
};

type AuthHandler = Box<
    dyn for<'a> Fn(Arc<WsIoServerConnection>, Option<&'a [u8]>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

type OnlyConnectionParamHandler = Arc<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<AuthHandler>,
    pub(crate) auth_timeout: Duration,
    pub(crate) middleware: Option<OnlyConnectionParamHandler>,
    pub(crate) on_connect_handler: Option<OnlyConnectionParamHandler>,
    pub(crate) on_ready_handler: Option<OnlyConnectionParamHandler>,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(super) path: String,
    pub(crate) websocket_config: WebSocketConfig,
}
