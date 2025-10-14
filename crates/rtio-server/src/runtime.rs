use crate::config::WsIoConfig;

#[derive(Clone, Debug)]
pub struct WsIoRuntime {
    pub(crate) config: WsIoConfig,
}

impl WsIoRuntime {
    pub fn new(config: WsIoConfig) -> Self {
        WsIoRuntime { config }
    }
}
