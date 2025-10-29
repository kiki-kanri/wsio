use std::time::Duration;

use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::core::packet::codecs::WsIoPacketCodec;

// Structs
#[derive(Debug)]
pub(crate) struct WsIoServerConfig {
    /// Maximum duration allowed for the auth handler to execute.
    ///
    /// Can be overridden by namespace-level configuration.
    pub(crate) auth_handler_timeout: Duration,

    /// Maximum duration to wait for the client to send the auth packet.
    ///
    /// Can be overridden by namespace-level configuration.
    pub(crate) auth_packet_timeout: Duration,

    pub(crate) broadcast_concurrency_limit: usize,

    /// Maximum duration allowed for middleware execution.
    ///
    /// Can be overridden by namespace-level configuration.
    pub(crate) middleware_execution_timeout: Duration,

    /// Maximum duration allowed for the on_close handler to execute.
    ///
    /// Can be overridden by namespace-level configuration.
    pub(crate) on_close_handler_timeout: Duration,

    /// Maximum duration allowed for the on_connect handler to execute.
    ///
    /// Can be overridden by namespace-level configuration.
    pub(crate) on_connect_handler_timeout: Duration,

    /// Can be overridden by namespace-level configuration.
    pub(crate) packet_codec: WsIoPacketCodec,

    pub(crate) request_path: String,

    /// Can be overridden by namespace-level configuration.
    pub(crate) websocket_config: WebSocketConfig,
}
