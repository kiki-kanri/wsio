use tower_layer::Layer;

use super::{
    inner::InnerWsIo,
    service::WsIoService,
};

pub struct WsIoLayer {
    inner: InnerWsIo,
}

impl WsIoLayer {
    pub fn new() -> Self {
        WsIoLayer {
            inner: InnerWsIo::new(),
        }
    }
}

impl<S> Layer<S> for WsIoLayer {
    type Service = WsIoService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WsIoService::with_client(inner)
    }
}
