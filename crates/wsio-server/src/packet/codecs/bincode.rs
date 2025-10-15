use ::bincode::{
    config::standard,
    serde::{
        decode_from_slice,
        encode_to_vec,
    },
};
use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::super::WsIoPacket;

pub(super) struct WsIoPacketBincodeCodec;

impl WsIoPacketBincodeCodec {
    #[inline]
    pub(super) fn encode<D: Serialize>(&self, packet: &WsIoPacket<D>) -> Result<Vec<u8>> {
        Ok(encode_to_vec(packet, standard())?)
    }

    #[inline]
    pub(super) fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        let (packet, _) = decode_from_slice(bytes, standard())?;
        Ok(packet)
    }
}
