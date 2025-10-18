use std::{
    pin::Pin,
    sync::Arc,
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
    pub(crate) fn new<H, Fut>(path: &str, on_connect_handler: H, runtime: Arc<WsIoServerRuntime>) -> Self
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let config = WsIoServerNamespaceConfig {
            auth_handler: None,
            on_connect_handler: Box::new(move |connection| Box::pin(on_connect_handler(connection))),
            packet_codec: runtime.config.default_packet_codec.clone(),
            path: path.into(),
        };

        Self { config, runtime }
    }

    // Public methods
    pub fn register(self) -> Result<Arc<WsIoServerNamespace>> {
        let namespace = Arc::new(WsIoServerNamespace::new(self.config, self.runtime.clone()));
        self.runtime.insert_namespace(namespace.clone())?;
        Ok(namespace)
    }

    pub fn with_auth<H, Fut, A: DeserializeOwned>(mut self, handler: H) -> WsIoServerNamespaceBuilder
    where
        H: Fn(Arc<WsIoServerConnection>, A) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let packet_codec = self.config.packet_codec.clone();
        let arc_handler = Arc::new(move |connection, bytes: Vec<u8>| {
            let handler = handler.clone();
            let packet_codec = packet_codec.clone();
            Box::pin(async move {
                let auth_data = packet_codec.decode_data::<A>(&bytes)?;
                handler(connection, auth_data).await
            }) as Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        });

        self.config.auth_handler = Some(arc_handler);
        self
    }

    pub fn with_packet_codec(mut self, packet_codec: WsIoPacketCodec) -> WsIoServerNamespaceBuilder {
        self.config.packet_codec = packet_codec;
        self
    }
}
