use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use super::{
    WsIoServerNamespace,
    config::WsIoServerNamespaceConfig,
};
use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoServerRuntime,
};

// Structs
pub struct WsIoServerNamespaceBuilder {
    config: WsIoServerNamespaceConfig,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespaceBuilder {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoServerRuntime>) -> Self {
        Self {
            config: WsIoServerNamespaceConfig {
                broadcast_concurrency_limit: runtime.config.broadcast_concurrency_limit,
                init_request_handler: None,
                init_request_handler_timeout: runtime.config.init_request_handler_timeout,
                init_response_handler: None,
                init_response_handler_timeout: runtime.config.init_response_handler_timeout,
                init_response_timeout: runtime.config.init_response_timeout,
                middleware: None,
                middleware_execution_timeout: runtime.config.middleware_execution_timeout,
                on_connect_handler: None,
                on_close_handler_timeout: runtime.config.on_close_handler_timeout,
                on_connect_handler_timeout: runtime.config.on_connect_handler_timeout,
                on_ready_handler: None,
                packet_codec: runtime.config.packet_codec,
                path: path.into(),
                websocket_config: runtime.config.websocket_config,
            },
            runtime,
        }
    }

    // Public methods
    pub fn broadcast_concurrency_limit(mut self, broadcast_concurrency_limit: usize) -> Self {
        self.config.broadcast_concurrency_limit = broadcast_concurrency_limit;
        self
    }

    pub fn middleware_execution_timeout(mut self, duration: Duration) -> Self {
        self.config.middleware_execution_timeout = duration;
        self
    }

    pub fn init_request_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.init_request_handler_timeout = duration;
        self
    }

    pub fn init_response_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.init_response_handler_timeout = duration;
        self
    }

    pub fn init_response_timeout(mut self, duration: Duration) -> Self {
        self.config.init_response_timeout = duration;
        self
    }

    pub fn on_close_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.on_close_handler_timeout = duration;
        self
    }

    pub fn on_connect<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_connect_handler = Some(Box::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn on_connect_handler_timeout(mut self, duration: Duration) -> Self {
        self.config.on_connect_handler_timeout = duration;
        self
    }

    pub fn on_ready<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_ready_handler = Some(Arc::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn register(self) -> Result<Arc<WsIoServerNamespace>> {
        let namespace = WsIoServerNamespace::new(self.config, self.runtime.clone());
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn websocket_config(mut self, websocket_config: WebSocketConfig) -> Self {
        self.config.websocket_config = websocket_config;
        self
    }

    pub fn websocket_config_mut<F: FnOnce(&mut WebSocketConfig)>(mut self, f: F) -> Self {
        f(&mut self.config.websocket_config);
        self
    }

    pub fn with_middleware<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.middleware = Some(Box::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn with_init_request<H, Fut, D>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D>>> + Send + 'static,
        D: Serialize + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.config.init_request_handler = Some(Box::new(move |connection, packet_codec| {
            let handler = handler.clone();
            Box::pin(async move {
                handler(connection)
                    .await?
                    .map(|data| packet_codec.encode_data(&data))
                    .transpose()
            })
        }));

        self
    }

    pub fn with_init_response<H, Fut, D>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>, Option<D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.config.init_response_handler = Some(Box::new(move |connection, bytes, packet_codec| {
            let handler = handler.clone();
            Box::pin(async move {
                handler(
                    connection,
                    bytes.map(|bytes| packet_codec.decode_data(bytes)).transpose()?,
                )
                .await
            })
        }));

        self
    }
}
