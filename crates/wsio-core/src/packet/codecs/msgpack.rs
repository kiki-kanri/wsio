use std::sync::LazyLock;

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
struct InnerPacket(Option<Vec<u8>>, Option<String>, WsIoPacketType);
pub(super) struct WsIoPacketMsgPackCodec;

static EMPTY_DATA_ENCODED: LazyLock<Vec<u8>> = LazyLock::new(|| to_vec(&()).unwrap());

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
    pub(super) fn encode(&self, packet: WsIoPacket) -> Result<Vec<u8>> {
        let inner_packet = InnerPacket(packet.data, packet.key, packet.r#type);
        Ok(to_vec(&inner_packet)?)
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
