use std::sync::Arc;

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
    pub(crate) fn new(mut namespace_url: Url) -> Result<Self> {
        if !matches!(namespace_url.scheme(), "ws" | "wss") {
            bail!("Invalid namespace URL scheme: {}", namespace_url.scheme());
        }

        let namespace = namespace_url.path();
        namespace_url.set_query(Some(&format!("namespace={}", namespace)));
        namespace_url.set_path("ws.io");
        Ok(Self {
            config: WsIoClientConfig {
                auth_handler: None,
                connect_url: namespace_url,
                packet_codec: WsIoPacketCodec::SerdeJson,
            },
        })
    }

    // Public methods
    pub fn auth<H, Fut, D>(mut self, handler: H) -> WsIoClientBuilder
    where
        H: Fn(Arc<WsIoClientConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D>>> + Send + 'static,
        D: Serialize,
    {
        let handler = Arc::new(handler);
        self.config.auth_handler = Some(Box::new(move |connection| {
            let handler = handler.clone();
            Box::pin(async move {
                Ok(match handler(connection).await? {
                    Some(data) => Some(self.config.packet_codec.encode_data(&data)?.to_vec()),
                    None => None,
                })
            })
        }));

        self
    }

    pub fn build(self) -> WsIoClient {
        WsIoClient(Arc::new(WsIoClientRuntime::new(self.config)))
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn request_path(mut self, request_path: impl AsRef<str>) -> Self {
        self.config.connect_url.set_path(request_path.as_ref());
        self
    }
}
