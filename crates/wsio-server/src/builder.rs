use std::time::Duration;

use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::{
    WsIoServer,
    config::WsIoServerConfig,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerBuilder {
    config: WsIoServerConfig,
}

impl WsIoServerBuilder {
    pub(crate) fn new() -> Self {
        Self {
            config: WsIoServerConfig {
                auth_handler_timeout: Duration::from_secs(3),
                auth_packet_timeout: Duration::from_secs(3),
                middleware_execution_timeout: Duration::from_secs(3),
                on_close_handler_timeout: Duration::from_secs(3),
                on_connect_handler_timeout: Duration::from_secs(3),
                packet_codec: WsIoPacketCodec::SerdeJson,
                request_path: "/ws.io".into(),
                websocket_config: WebSocketConfig::default()
                    .max_frame_size(Some(8 * 1024 * 1024))
                    .max_message_size(Some(16 * 1024 * 1024))
                    .max_write_buffer_size(2 * 1024 * 1024)
                    .read_buffer_size(8 * 1024)
                    .write_buffer_size(8 * 1024),
            },
        }
    }

    // Public methods
    pub fn auth_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.auth_handler_timeout = duration;
        self
    }

    pub fn auth_packet_timeout(mut self, duration: Duration) -> Self {
        self.config.auth_packet_timeout = duration;
        self
    }

    pub fn build(self) -> WsIoServer {
        WsIoServer(WsIoServerRuntime::new(self.config))
    }

    pub fn middleware_execution_timeout(mut self, duration: Duration) -> Self {
        self.config.middleware_execution_timeout = duration;
        self
    }

    pub fn on_close_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.on_close_handler_timeout = duration;
        self
    }

    pub fn on_connect_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.on_connect_handler_timeout = duration;
        self
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.request_path = request_path.as_ref().into();
        self
    }

    pub fn websocket_config(mut self, websocket_config: WebSocketConfig) -> Self {
        self.config.websocket_config = websocket_config;
        self
    }

    pub fn websocket_config_mut<F: FnOnce(&mut WebSocketConfig)>(mut self, f: F) -> Self {
        f(&mut self.config.websocket_config);
        self
    }
}
