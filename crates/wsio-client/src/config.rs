use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

use crate::{
    connection::WsIoClientConnection,
    core::packet::codecs::WsIoPacketCodec,
};

type AuthHandler = Box<
    dyn Fn(Arc<WsIoClientConnection>) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

type OnlyConnectionParamHandler = Arc<
    dyn Fn(Arc<WsIoClientConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) struct WsIoClientConfig {
    pub(crate) auth_handler: Option<AuthHandler>,
    pub(crate) connect_url: Url,
    pub(crate) init_timeout: Duration,
    pub(crate) on_connection_close_handler: Option<OnlyConnectionParamHandler>,
    pub(crate) on_connection_ready_handler: Option<OnlyConnectionParamHandler>,
    pub(crate) packet_codec: WsIoPacketCodec,
    pub(crate) reconnection_delay: Duration,
    pub(crate) ready_timeout: Duration,
    pub(crate) websocket_config: WebSocketConfig,
}
