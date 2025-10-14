use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct WsIoConfig {
    pub request_path: Cow<'static, str>,
}

impl Default for WsIoConfig {
    fn default() -> Self {
        Self {
            request_path: "/ws.io".into(),
        }
    }
}
