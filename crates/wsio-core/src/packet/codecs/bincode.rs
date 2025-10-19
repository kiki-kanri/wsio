use std::sync::LazyLock;

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
struct InnerPacket(Option<Vec<u8>>, Option<String>, WsIoPacketType);
pub(super) struct WsIoPacketBincodeCodec;

static EMPTY_DATA_ENCODED: LazyLock<Vec<u8>> = LazyLock::new(|| encode_to_vec((), standard()).unwrap());

impl WsIoPacketBincodeCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        let (inner_packet, _) = decode_from_slice::<InnerPacket, _>(bytes, standard())?;
        Ok(WsIoPacket {
            data: inner_packet.0,
            key: inner_packet.1,
            r#type: inner_packet.2,
        })
    }

    #[inline]
    pub(super) fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        let (data, _) = decode_from_slice::<D, _>(bytes, standard())?;
        Ok(data)
    }

    #[inline]
    pub(super) fn empty_data_encoded(&self) -> &[u8] {
        EMPTY_DATA_ENCODED.as_ref()
    }

    // TODO: &WsIoPacket
    #[inline]
    pub(super) fn encode(&self, packet: WsIoPacket) -> Result<Vec<u8>> {
        let inner_packet = InnerPacket(packet.data, packet.key, packet.r#type);
        Ok(encode_to_vec(inner_packet, standard())?)
    }

    #[inline]
    pub(super) fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        Ok(encode_to_vec(data, standard())?)
    }
}
