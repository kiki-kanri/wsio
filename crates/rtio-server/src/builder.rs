use super::{
    WsIo,
    inner::InnerWsIo,
    layer::WsIoLayer,
};

pub struct WsIoBuilder {}

impl WsIoBuilder {
    pub fn new() -> Self {
        WsIoBuilder {}
    }

    pub fn build_layer(&self) -> (WsIoLayer, WsIo) {
        (WsIoLayer::new(), WsIo(InnerWsIo::new()))
    }
}
