use anyhow::Result;
use postcard::{
    from_bytes,
    to_allocvec,
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

// Structs
#[derive(Deserialize, Serialize)]
struct InnerPacket(Option<Vec<u8>>, Option<String>, WsIoPacketType);
pub(super) struct WsIoPacketPostcardCodec;

impl WsIoPacketPostcardCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        let inner_packet = from_bytes::<InnerPacket>(bytes)?;
        Ok(WsIoPacket {
            data: inner_packet.0,
            key: inner_packet.1,
            r#type: inner_packet.2,
        })
    }

    #[inline]
    pub(super) fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        Ok(from_bytes::<D>(bytes)?)
    }

    #[inline]
    pub(super) fn encode(&self, packet: WsIoPacket) -> Result<Vec<u8>> {
        Ok(to_allocvec(&InnerPacket(packet.data, packet.key, packet.r#type))?)
    }

    #[inline]
    pub(super) fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        Ok(to_allocvec(data)?)
    }
}
