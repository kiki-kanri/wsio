use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::{
    Result,
    bail,
};
use serde::Serialize;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use url::Url;

use crate::{
    WsIoClient,
    config::WsIoClientConfig,
    connection::WsIoClientConnection,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoClientRuntime,
};

// Structs
pub struct WsIoClientBuilder {
    config: WsIoClientConfig,
    connect_url: Url,
}

impl WsIoClientBuilder {
    pub(crate) fn new(mut url: Url) -> Result<Self> {
        if !matches!(url.scheme(), "ws" | "wss") {
            bail!("Invalid URL scheme: {}", url.scheme());
        }

        url.set_query(Some(&format!("namespace={}", Self::normalize_url_path(url.path()))));
        url.set_path("ws.io");
        Ok(Self {
            config: WsIoClientConfig {
                auth_handler: None,
                auth_handler_timeout: Duration::from_secs(3),
                init_packet_timeout: Duration::from_secs(3),
                on_connection_close_handler: None,
                on_connection_close_handler_timeout: Duration::from_secs(2),
                on_connection_ready_handler: None,
                packet_codec: WsIoPacketCodec::SerdeJson,
                ready_packet_timeout: Duration::from_secs(3),
                reconnection_delay: Duration::from_secs(1),
                websocket_config: WebSocketConfig::default()
                    .max_frame_size(Some(8 * 1024 * 1024))
                    .max_message_size(Some(16 * 1024 * 1024))
                    .max_write_buffer_size(2 * 1024 * 1024)
                    .read_buffer_size(8 * 1024)
                    .write_buffer_size(8 * 1024),
            },
            connect_url: url,
        })
    }

    // Private methods
    fn normalize_url_path(path: &str) -> String {
        format!(
            "/{}",
            path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("/")
        )
    }

    // Public methods
    pub fn auth<H, Fut, D>(mut self, handler: H) -> WsIoClientBuilder
    where
        D: Serialize,
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<D>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.config.auth_handler = Some(Box::new(move |connection, packet_codec| {
            let handler = handler.clone();
            Box::pin(async move { packet_codec.encode_data(&handler(connection).await?) })
        }));

        self
    }

    pub fn auth_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.auth_handler_timeout = duration;
        self
    }

    pub fn build(self) -> WsIoClient {
        WsIoClient(WsIoClientRuntime::new(self.config, self.connect_url))
    }

    pub fn init_packet_timeout(mut self, duration: Duration) -> Self {
        self.config.init_packet_timeout = duration;
        self
    }

    pub fn on_connection_close<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_connection_close_handler = Some(Box::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn on_connection_close_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.on_connection_close_handler_timeout = duration;
        self
    }

    pub fn on_connection_ready<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_connection_ready_handler = Some(Arc::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn ready_packet_timeout(mut self, duration: Duration) -> Self {
        self.config.ready_packet_timeout = duration;
        self
    }

    pub fn reconnection_delay(mut self, delay: Duration) -> Self {
        self.config.reconnection_delay = delay;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.connect_url
            .set_path(&Self::normalize_url_path(request_path.as_ref()));

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
