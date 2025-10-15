use std::sync::Arc;

use anyhow::Result;

mod builder;
mod config;
mod context;
mod layer;
mod namespace;
pub mod packet;
mod runtime;
mod service;

use builder::WsIoBuilder;
use namespace::{
    WsIoNamespace,
    builder::WsIoNamespaceBuilder,
};
use runtime::WsIoRuntime;

#[derive(Clone)]
pub struct WsIo(Arc<WsIoRuntime>);

impl WsIo {
    pub fn builder() -> WsIoBuilder {
        WsIoBuilder::new()
    }

    #[inline]
    pub fn of(&self, path: impl AsRef<str>) -> Option<Arc<WsIoNamespace>> {
        self.0.get_namespace(path)
    }

    #[inline]
    pub fn ns(&self, path: impl AsRef<str>) -> Result<WsIoNamespaceBuilder> {
        self.0.new_namespace_builder(path)
    }
}
