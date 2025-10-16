use ::bincode::{
    config::standard,
    serde::{
        decode_from_slice,
        encode_to_vec,
    },
};
use anyhow::Result;
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
pub(super) struct WsIoPacketBincodeCodec;

impl WsIoPacketBincodeCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn encode<D: Serialize>(&self, packet: WsIoPacket<D>) -> Result<Vec<u8>> {
        let inner_packet = InnerPacket(packet.data, packet.key, packet.r#type);
        Ok(encode_to_vec(inner_packet, standard())?)
    }

    #[inline]
    pub(super) fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        let (inner_packet, _) = decode_from_slice::<InnerPacket<D>, _>(bytes, standard())?;
        Ok(WsIoPacket {
            data: inner_packet.0,
            key: inner_packet.1,
            r#type: inner_packet.2,
        })
    }
}
