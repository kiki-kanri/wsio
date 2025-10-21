use std::time::Duration;

use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::core::packet::codecs::WsIoPacketCodec;

#[derive(Debug)]
pub(crate) struct WsIoServerConfig {
    pub(crate) auth_timeout: Duration,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(crate) request_path: String,
    pub(crate) websocket_config: WebSocketConfig,
}
