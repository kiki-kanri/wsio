use anyhow::Result;
use rmp_serde::{
    from_slice,
    to_vec,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::super::{
    InnerPacket,
    InnerPacketRef,
    WsIoPacket,
};

// Structs
pub(super) struct WsIoPacketMsgPackCodec;

impl WsIoPacketMsgPackCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        let inner_packet = from_slice::<InnerPacket>(bytes)?;
        Ok(WsIoPacket {
            data: inner_packet.0,
            key: inner_packet.1,
            r#type: inner_packet.2,
        })
    }

    #[inline]
    pub(super) fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        Ok(from_slice(bytes)?)
    }

    #[inline]
    pub(super) fn encode(&self, packet: &WsIoPacket) -> Result<Vec<u8>> {
        Ok(to_vec(&InnerPacketRef(&packet.data, &packet.key, &packet.r#type))?)
    }

    #[inline]
    pub(super) fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        Ok(to_vec(data)?)
    }
}
