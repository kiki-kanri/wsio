use std::sync::Arc;

use anyhow::Result;
pub use wsio_core as core;

mod builder;
mod config;
pub mod connection;
mod layer;
mod namespace;
mod runtime;
mod service;

use crate::{
    builder::WsIoServerBuilder,
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
    // Public methods
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
    pub fn namespace_count(&self) -> usize {
        self.0.namespace_count()
    }

    #[inline]
    pub fn new_namespace_builder(&self, path: impl AsRef<str>) -> Result<WsIoServerNamespaceBuilder> {
        self.0.new_namespace_builder(path.as_ref())
    }

    #[inline]
    pub fn of(&self, path: impl AsRef<str>) -> Option<Arc<WsIoServerNamespace>> {
        self.0.get_namespace(path.as_ref())
    }
}
