use std::sync::Arc;

use tower_layer::Layer;

use crate::{
    runtime::WsIoServerRuntime,
    service::WsIoServerService,
};

// Structs
#[derive(Clone)]
pub struct WsIoServerLayer {
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerLayer {
    pub(crate) fn new(runtime: Arc<WsIoServerRuntime>) -> Self {
        Self { runtime }
    }
}

impl<S> Layer<S> for WsIoServerLayer {
    type Service = WsIoServerService<S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        WsIoServerService::new(inner, self.runtime.clone())
    }
}
