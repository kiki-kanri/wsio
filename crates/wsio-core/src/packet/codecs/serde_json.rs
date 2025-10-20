use std::sync::LazyLock;

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

static EMPTY_DATA_ENCODED: LazyLock<Vec<u8>> = LazyLock::new(|| to_vec(&()).unwrap());

impl WsIoPacketSerdeJsonCodec {
    pub(super) const IS_TEXT: bool = true;

    #[inline]
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        Ok(from_slice::<WsIoPacket>(bytes)?)
    }

    #[inline]
    pub(super) fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        Ok(from_slice(bytes)?)
    }

    #[inline]
    pub(super) fn encode(&self, packet: &WsIoPacket) -> Result<Vec<u8>> {
        Ok(to_vec(packet)?)
    }

    #[inline]
    pub(super) fn empty_data_encoded(&self) -> &[u8] {
        EMPTY_DATA_ENCODED.as_ref()
    }

    #[inline]
    pub(super) fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        Ok(to_vec(data)?)
    }
}
