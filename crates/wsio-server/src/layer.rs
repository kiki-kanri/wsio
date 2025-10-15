use std::sync::Arc;

use tower_layer::Layer;

use crate::{
    runtime::WsIoRuntime,
    service::WsIoService,
};

#[derive(Clone, Debug)]
pub struct WsIoLayer {
    runtime: Arc<WsIoRuntime>,
}

impl WsIoLayer {
    pub(crate) fn new(runtime: Arc<WsIoRuntime>) -> Self {
        Self { runtime }
    }
}

impl<S> Layer<S> for WsIoLayer {
    type Service = WsIoService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WsIoService::new(inner, self.runtime.clone())
    }
}
