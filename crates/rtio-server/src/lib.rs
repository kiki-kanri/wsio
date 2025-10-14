mod builder;
mod config;
mod layer;
mod namespace;
mod runtime;
pub mod service;

use builder::WsIoBuilder;
use runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIo(WsIoRuntime);

impl WsIo {
    pub fn builder() -> WsIoBuilder {
        WsIoBuilder::new()
    }
}
