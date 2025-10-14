use crate::runtime::WsIoRuntime;

#[derive(Clone)]
pub struct WsIoService<S> {
    inner: S,
    runtime: WsIoRuntime,
}

impl<S> WsIoService<S> {
    pub fn new(inner: S, runtime: WsIoRuntime) -> Self {
        Self { inner, runtime }
    }

    pub fn runtime(&self) -> &WsIoRuntime {
        &self.runtime
    }
}
