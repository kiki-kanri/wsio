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
    core::{
        packet::codecs::WsIoPacketCodec,
        types::{
            ArcAsyncUnaryResultHandler,
            BoxAsyncUnaryResultHandler,
        },
    },
};

type AuthHandler = Box<
    dyn for<'a> Fn(
            Arc<WsIoClientConnection>,
            &'a WsIoPacketCodec,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) struct WsIoClientConfig {
    pub(crate) auth_handler: Option<AuthHandler>,

    /// Maximum duration allowed for the auth handler to execute.
    pub(crate) auth_handler_timeout: Duration,

    pub(crate) connect_url: Url,

    /// Maximum duration to wait for the client to send the init packet.
    pub(crate) init_packet_timeout: Duration,

    pub(crate) on_connection_close_handler: Option<BoxAsyncUnaryResultHandler<WsIoClientConnection>>,

    /// Maximum duration allowed for the on_connection_close handler to execute.
    pub(crate) on_connection_close_handler_timeout: Duration,

    pub(crate) on_connection_ready_handler: Option<ArcAsyncUnaryResultHandler<WsIoClientConnection>>,

    pub(crate) packet_codec: WsIoPacketCodec,

    /// Maximum duration to wait for the client to send the ready packet.
    pub(crate) ready_packet_timeout: Duration,

    pub(crate) reconnection_delay: Duration,

    pub(crate) websocket_config: WebSocketConfig,
}
