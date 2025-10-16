use std::sync::Arc;

use anyhow::Result;
pub use wsioxide_core as core;

mod builder;
mod config;
pub mod connection;
mod layer;
mod namespace;
mod runtime;
mod service;

use crate::{
    builder::WsIoServerBuilder,
    connection::WsIoServerConnection,
    layer::WsIoServerLayer,
    namespace::{
        WsIoServerNamespace,
        builder::WsIoServerNamespaceBuilder,
    },
    runtime::WsIoServerRuntime,
};

#[derive(Clone)]
pub struct WsIoServer(Arc<WsIoServerRuntime>);

impl WsIoServer {
    pub fn builder() -> WsIoServerBuilder {
        WsIoServerBuilder::new()
    }

    #[inline]
    pub fn connection_count(&self) -> usize {
        self.0.connection_count()
    }

    pub fn layer(&self) -> WsIoServerLayer {
        WsIoServerLayer::new(self.0.clone())
    }

    #[inline]
    pub fn new_namespace_builder<H, Fut>(
        &self,
        path: impl AsRef<str>,
        on_connect_handler: H,
    ) -> Result<WsIoServerNamespaceBuilder>
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        self.0.new_namespace_builder(path.as_ref(), on_connect_handler)
    }

    #[inline]
    pub fn of(&self, path: impl AsRef<str>) -> Option<Arc<WsIoServerNamespace>> {
        self.0.get_namespace(path.as_ref())
    }
}
