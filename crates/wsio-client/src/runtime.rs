use crate::config::WsIoClientConfig;

pub(crate) struct WsIoClientRuntime {
    config: WsIoClientConfig,
}

impl WsIoClientRuntime {
    pub(crate) fn new(config: WsIoClientConfig) -> Self {
        Self { config }
    }
}
