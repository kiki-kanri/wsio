use anyhow::Result;
use rmp_serde::{
    from_slice,
    to_vec,
};
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};

use super::super::{
    WsIoPacket,
    WsIoPacketType,
};

#[derive(Deserialize, Serialize)]
struct InnerPacket<D>(Option<D>, Option<String>, WsIoPacketType);
pub(super) struct WsIoPacketMsgPackCodec;

impl WsIoPacketMsgPackCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn encode<D: Serialize>(&self, packet: WsIoPacket<D>) -> Result<Vec<u8>> {
        let inner_packet = InnerPacket(packet.data, packet.key, packet.r#type);
        Ok(to_vec(&inner_packet)?)
    }

    #[inline]
    pub(super) fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        let inner_packet = from_slice::<InnerPacket<D>>(bytes)?;
        Ok(WsIoPacket {
            data: inner_packet.0,
            key: inner_packet.1,
            r#type: inner_packet.2,
        })
    }
}
