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

use builder::WsIoServerBuilder;
use namespace::{
    WsIoServerNamespace,
    builder::WsIoServerNamespaceBuilder,
};
use runtime::WsIoServerRuntime;

#[derive(Clone)]
pub struct WsIoServer(Arc<WsIoServerRuntime>);

impl WsIoServer {
    pub fn builder() -> WsIoServerBuilder {
        WsIoServerBuilder::new()
    }

    #[inline]
    pub fn of(&self, path: impl AsRef<str>) -> Option<Arc<WsIoServerNamespace>> {
        self.0.get_namespace(path)
    }

    #[inline]
    pub fn ns(&self, path: impl AsRef<str>) -> Result<WsIoServerNamespaceBuilder> {
        self.0.new_namespace_builder(path)
    }
}
