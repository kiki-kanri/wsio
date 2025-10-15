use std::sync::Arc;

use crate::runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIoNamespace {
    path: String,
    runtime: Arc<WsIoRuntime>,
}

impl WsIoNamespace {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoRuntime>) -> WsIoNamespace {
        WsIoNamespace {
            path: path.to_string(),
            runtime,
        }
    }
}
