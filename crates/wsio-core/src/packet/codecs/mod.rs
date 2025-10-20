use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

#[cfg(feature = "packet-codec-bincode")]
mod bincode;
#[cfg(feature = "packet-codec-cbor")]
mod cbor;
#[cfg(feature = "packet-codec-msgpack")]
mod msgpack;
mod serde_json;
#[cfg(feature = "packet-codec-sonic-rs")]
mod sonic_rs;

#[cfg(feature = "packet-codec-bincode")]
use self::bincode::WsIoPacketBincodeCodec;
#[cfg(feature = "packet-codec-cbor")]
use self::cbor::WsIoPacketCborCodec;
#[cfg(feature = "packet-codec-msgpack")]
use self::msgpack::WsIoPacketMsgPackCodec;
use self::serde_json::WsIoPacketSerdeJsonCodec;
#[cfg(feature = "packet-codec-sonic-rs")]
use self::sonic_rs::WsIoPacketSonicRsCodec;
use super::WsIoPacket;

#[derive(Clone, Copy, Debug)]
pub enum WsIoPacketCodec {
    #[cfg(feature = "packet-codec-bincode")]
    Bincode,
    #[cfg(feature = "packet-codec-cbor")]
    Cbor,
    #[cfg(feature = "packet-codec-msgpack")]
    MsgPack,
    SerdeJson,
    #[cfg(feature = "packet-codec-sonic-rs")]
    SonicRs,
}

impl WsIoPacketCodec {
    #[inline]
    pub fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.decode(bytes),
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec.decode(bytes),
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.decode(bytes),
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.decode(bytes),
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.decode(bytes),
        }
    }

    #[inline]
    pub fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.decode_data(bytes),
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec.decode_data(bytes),
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.decode_data(bytes),
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.decode_data(bytes),
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.decode_data(bytes),
        }
    }

    #[inline]
    pub fn encode(&self, packet: &WsIoPacket) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.encode(packet.clone()),
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec.encode(packet),
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.encode(packet.clone()),
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.encode(packet),
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.encode(packet),
        }
    }

    #[inline]
    pub fn empty_data_encoded(&self) -> &[u8] {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.empty_data_encoded(),
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec.empty_data_encoded(),
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.empty_data_encoded(),
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.empty_data_encoded(),
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.empty_data_encoded(),
        }
    }

    #[inline]
    pub fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.encode_data(data),
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec.encode_data(data),
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.encode_data(data),
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.encode_data(data),
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.encode_data(data),
        }
    }

    pub fn is_text(&self) -> bool {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec::IS_TEXT,
            #[cfg(feature = "packet-codec-cbor")]
            Self::Cbor => WsIoPacketCborCodec::IS_TEXT,
            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec::IS_TEXT,
            Self::SerdeJson => WsIoPacketSerdeJsonCodec::IS_TEXT,
            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec::IS_TEXT,
        }
    }
}
