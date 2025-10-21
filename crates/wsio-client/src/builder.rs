use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::{
    Result,
    bail,
};
use serde::Serialize;
use url::Url;

use crate::{
    WsIoClient,
    config::WsIoClientConfig,
    connection::WsIoClientConnection,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoClientRuntime,
};

pub struct WsIoClientBuilder {
    config: WsIoClientConfig,
}

impl WsIoClientBuilder {
    pub(crate) fn new(mut url: Url) -> Result<Self> {
        if !matches!(url.scheme(), "ws" | "wss") {
            bail!("Invalid URL scheme: {}", url.scheme());
        }

        let namespace = {
            let path = url.path().trim_matches('/');
            let mut normalized = String::new();
            let mut last_slash = false;
            for char in path.chars() {
                if char == '/' {
                    if !last_slash {
                        normalized.push('/');
                        last_slash = true;
                    }
                } else {
                    normalized.push(char);
                    last_slash = false;
                }
            }

            normalized
        };

        url.set_query(Some(&format!("namespace=/{}", namespace)));
        url.set_path("ws.io");
        Ok(Self {
            config: WsIoClientConfig {
                auth_handler: None,
                connect_url: url,
                init_timeout: Duration::from_secs(3),
                on_connection_close_handler: None,
                on_connection_ready_handler: None,
                packet_codec: WsIoPacketCodec::SerdeJson,
                ready_timeout: Duration::from_secs(3),
                reconnection_delay: Duration::from_secs(1),
            },
        })
    }

    // Public methods
    pub fn auth<D, H, Fut>(mut self, handler: H) -> WsIoClientBuilder
    where
        D: Serialize,
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D>>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.config.auth_handler = Some(Box::new(move |connection| {
            let handler = handler.clone();
            Box::pin(async move {
                handler(connection)
                    .await?
                    .map(|data| self.config.packet_codec.encode_data(&data))
                    .transpose()
            })
        }));

        self
    }

    pub fn build(self) -> WsIoClient {
        WsIoClient(Arc::new(WsIoClientRuntime::new(self.config)))
    }

    pub fn init_timeout(mut self, duration: Duration) -> Self {
        self.config.init_timeout = duration;
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

    pub fn on_connection_ready<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_connection_ready_handler = Some(Box::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn ready_timeout(mut self, duration: Duration) -> Self {
        self.config.ready_timeout = duration;
        self
    }

    pub fn reconnection_delay(mut self, delay: Duration) -> Self {
        self.config.reconnection_delay = delay;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.connect_url.set_path(request_path.as_ref());
        self
    }
}
