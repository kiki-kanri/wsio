use std::{
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

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

// Types
type InitHandler = Box<
    dyn for<'a> Fn(
            Arc<WsIoClientConnection>,
            Option<&'a [u8]>,
            &'a WsIoPacketCodec,
        ) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

// Structs
pub(crate) struct WsIoClientConfig {
    pub(crate) init_handler: Option<InitHandler>,

    /// Maximum duration allowed for the init handler to execute.
    pub(crate) init_handler_timeout: Duration,

    /// Maximum duration to wait for the server to send the init packet.
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
