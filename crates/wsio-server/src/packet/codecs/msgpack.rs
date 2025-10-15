use anyhow::Result;
use rmp_serde::{
    from_slice,
    to_vec,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::super::WsIoPacket;

pub(super) struct WsIoPacketMsgPackCodec;

impl WsIoPacketMsgPackCodec {
    #[inline]
    pub(super) fn encode<D: Serialize>(&self, packet: &WsIoPacket<D>) -> Result<Vec<u8>> {
        Ok(to_vec(packet)?)
    }

    #[inline]
    pub(super) fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        Ok(from_slice::<WsIoPacket<D>>(bytes)?)
    }
}
