use tower_layer::Layer;

use crate::{
    runtime::WsIoRuntime,
    service::WsIoService,
};

#[derive(Clone, Debug)]
pub struct WsIoLayer {
    runtime: WsIoRuntime,
}

impl WsIoLayer {
    pub fn new(runtime: WsIoRuntime) -> Self {
        Self { runtime }
    }
}

impl<S: Clone> Layer<S> for WsIoLayer {
    type Service = WsIoService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WsIoService::new(inner, self.runtime.clone())
    }
}
