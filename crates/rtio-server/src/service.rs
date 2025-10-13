#[derive(Clone)]
pub struct WsIoService<S> {
    inner: S,
}

impl<S> WsIoService<S> {
    pub fn with_client(inner: S) -> Self {
        Self { inner }
    }
}
