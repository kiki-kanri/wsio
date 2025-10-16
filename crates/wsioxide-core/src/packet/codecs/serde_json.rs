use ::serde_json::{
    from_slice,
    to_vec,
};
use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::super::WsIoPacket;

pub(super) struct WsIoPacketSerdeJsonCodec;

impl WsIoPacketSerdeJsonCodec {
    pub(super) const IS_TEXT: bool = true;

    #[inline]
    pub(super) fn encode<D: Serialize>(&self, packet: WsIoPacket<D>) -> Result<Vec<u8>> {
        Ok(to_vec(&packet)?)
    }

    #[inline]
    pub(super) fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        Ok(from_slice::<WsIoPacket<D>>(bytes)?)
    }
}
