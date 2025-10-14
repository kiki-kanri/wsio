use crate::{
    WsIo,
    layer::WsIoLayer,
    runtime::WsIoRuntime,
};

pub struct WsIoBuilder {}

impl WsIoBuilder {
    pub fn new() -> Self {
        WsIoBuilder {}
    }

    pub fn build_layer(&self) -> (WsIoLayer, WsIo) {
        let runtime = WsIoRuntime::new();
        (WsIoLayer::new(runtime.clone()), WsIo(runtime))
    }
}
