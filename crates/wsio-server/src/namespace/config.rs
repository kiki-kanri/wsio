use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::{
    connection::WsIoServerConnection,
    core::{
        packet::codecs::WsIoPacketCodec,
        types::{
            ArcAsyncUnaryResultHandler,
            BoxAsyncUnaryResultHandler,
        },
    },
};

// Types
type AuthHandler = Box<
    dyn for<'a> Fn(
            Arc<WsIoServerConnection>,
            &'a [u8],
            &'a WsIoPacketCodec,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

// Structs
pub(crate) struct WsIoServerNamespaceConfig {
    pub(crate) auth_handler: Option<AuthHandler>,

    /// Maximum duration allowed for the auth handler to execute.
    pub(crate) auth_handler_timeout: Duration,

    /// Maximum duration to wait for the client to send the auth packet.
    pub(crate) auth_packet_timeout: Duration,

    /// Maximum number of concurrent broadcast operations.
    pub(crate) broadcast_concurrency_limit: usize,

    pub(crate) middleware: Option<BoxAsyncUnaryResultHandler<WsIoServerConnection>>,

    /// Maximum duration allowed for middleware execution.
    pub(crate) middleware_execution_timeout: Duration,

    /// Maximum duration allowed for the on_close handler to execute.
    pub(crate) on_close_handler_timeout: Duration,

    /// Maximum duration allowed for the on_connect handler to execute.
    pub(crate) on_connect_handler_timeout: Duration,

    pub(crate) on_connect_handler: Option<BoxAsyncUnaryResultHandler<WsIoServerConnection>>,

    pub(crate) on_ready_handler: Option<ArcAsyncUnaryResultHandler<WsIoServerConnection>>,

    pub(crate) packet_codec: WsIoPacketCodec,

    pub(super) path: String,

    pub(crate) websocket_config: WebSocketConfig,
}
