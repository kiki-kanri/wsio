use crate::runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIoNamespace {
    path: String,
    runtime: WsIoRuntime,
}

impl WsIoNamespace {
    pub(crate) fn new(path: &str, runtime: WsIoRuntime) -> WsIoNamespace {
        WsIoNamespace {
            path: path.to_string(),
            runtime,
        }
    }
}
