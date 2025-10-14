use std::borrow::Cow;

use crate::packet::codecs::WsIoPacketCodec;

#[derive(Clone, Debug)]
pub(crate) struct WsIoConfig {
    pub(crate) default_codec: WsIoPacketCodec,
    pub(crate) request_path: Cow<'static, str>,
}

impl Default for WsIoConfig {
    fn default() -> Self {
        Self {
            default_codec: WsIoPacketCodec::SerdeJson,
            request_path: "/ws.io".into(),
        }
    }
}
