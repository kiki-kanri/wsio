use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use serde::de::DeserializeOwned;

use super::{
    WsIoServerNamespace,
    config::WsIoServerNamespaceConfig,
};
use crate::{
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
    runtime::WsIoServerRuntime,
};

pub struct WsIoServerNamespaceBuilder {
    config: WsIoServerNamespaceConfig,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespaceBuilder {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoServerRuntime>) -> Self {
        Self {
            config: WsIoServerNamespaceConfig {
                auth_handler: None,
                auth_timeout: runtime.config.auth_timeout,
                middleware: None,
                on_connect_handler: None,
                on_ready_handler: None,
                packet_codec: runtime.config.packet_codec,
                path: path.into(),
            },
            runtime,
        }
    }

    // Public methods
    pub fn auth_timeout(mut self, duration: Duration) -> Self {
        self.config.auth_timeout = duration;
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

    pub fn on_ready<H, Fut>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.config.on_ready_handler = Some(Box::new(move |connection| Box::pin(handler(connection))));
        self
    }

    pub fn packet_codec(mut self, packet_codec: WsIoPacketCodec) -> Self {
        self.config.packet_codec = packet_codec;
        self
    }

    pub fn register(self) -> Result<Arc<WsIoServerNamespace>> {
        let namespace = Arc::new(WsIoServerNamespace::new(self.config, self.runtime.clone()));
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn with_auth<H, Fut, D>(mut self, handler: H) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>, Option<&D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + 'static,
    {
        let handler = Arc::new(handler);
        self.config.auth_handler = Some(Box::new(move |connection, bytes: Option<&[u8]>| {
            let handler = handler.clone();
            Box::pin(async move {
                let auth_data = match bytes {
                    Some(bytes) => Some(self.config.packet_codec.decode_data(bytes)?),
                    None => None,
                };

                handler(connection, auth_data.as_ref()).await
            })
        }));

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
}
