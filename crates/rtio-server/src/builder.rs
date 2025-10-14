use std::borrow::Cow;

use crate::{
    WsIo,
    config::WsIoConfig,
    layer::WsIoLayer,
    runtime::WsIoRuntime,
};

pub struct WsIoBuilder {
    config: WsIoConfig,
}

impl WsIoBuilder {
    pub fn new() -> Self {
        WsIoBuilder {
            config: WsIoConfig::default(),
        }
    }

    pub fn build_layer(&self) -> (WsIoLayer, WsIo) {
        let runtime = WsIoRuntime::new(self.config.clone());
        (WsIoLayer::new(runtime.clone()), WsIo(runtime))
    }

    pub fn request_path(mut self, request_path: impl Into<Cow<'static, str>>) -> Self {
        self.config.request_path = request_path.into();
        self
    }
}
