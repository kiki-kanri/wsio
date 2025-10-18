use std::time::Duration;

use crate::core::packet::codecs::WsIoPacketCodec;

#[derive(Debug)]
pub(crate) struct WsIoServerConfig {
    pub(crate) auth_timeout: Duration,
    pub(crate) default_packet_codec: WsIoPacketCodec,
    pub(crate) request_path: String,
}
