mod builder;
mod inner;
mod layer;
mod namespace;
mod service;

use builder::WsIoBuilder;
use inner::InnerWsIo;

#[derive(Clone, Debug)]
pub struct WsIo(InnerWsIo);

impl WsIo {
    pub fn builder() -> WsIoBuilder {
        WsIoBuilder::new()
    }
}
