use std::borrow::Cow;

use anyhow::Result;

mod builder;
mod config;
mod layer;
mod namespace;
pub mod packet;
mod runtime;
mod service;

use builder::WsIoBuilder;
use namespace::WsIoNamespace;
use runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIo(WsIoRuntime);

impl WsIo {
    pub fn builder() -> WsIoBuilder {
        WsIoBuilder::new()
    }

    #[inline]
    pub fn ns(&self, path: impl Into<Cow<'static, str>>) -> Result<WsIoNamespace> {
        Ok(self.0.add_ns(path.into())?)
    }
}
