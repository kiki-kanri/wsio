use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

#[cfg(feature = "packet-codec-bincode")]
mod bincode;
#[cfg(feature = "packet-codec-msgpack")]
mod msgpack;
#[cfg(feature = "packet-codec-serde-json")]
mod serde_json;
#[cfg(feature = "packet-codec-sonic-rs")]
mod sonic_rs;

#[cfg(feature = "packet-codec-bincode")]
use self::bincode::WsIoPacketBincodeCodec;
#[cfg(feature = "packet-codec-msgpack")]
use self::msgpack::WsIoPacketMsgPackCodec;
#[cfg(feature = "packet-codec-serde-json")]
use self::serde_json::WsIoPacketSerdeJsonCodec;
#[cfg(feature = "packet-codec-sonic-rs")]
use self::sonic_rs::WsIoPacketSonicRsCodec;
use super::WsIoPacket;

#[derive(Clone, Debug)]
pub enum WsIoPacketCodec {
    #[cfg(feature = "packet-codec-bincode")]
    Bincode,

    #[cfg(feature = "packet-codec-msgpack")]
    MsgPack,

    #[cfg(feature = "packet-codec-serde-json")]
    SerdeJson,

    #[cfg(feature = "packet-codec-sonic-rs")]
    SonicRs,
}

impl WsIoPacketCodec {
    #[inline]
    fn encode<D: Serialize>(&self, packet: &WsIoPacket<D>) -> Result<Vec<u8>> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.encode(packet),

            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.encode(packet),

            #[cfg(feature = "packet-codec-serde-json")]
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.encode(packet),

            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.encode(packet),
        }
    }

    #[inline]
    fn decode<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<WsIoPacket<D>> {
        match self {
            #[cfg(feature = "packet-codec-bincode")]
            Self::Bincode => WsIoPacketBincodeCodec.decode(bytes),

            #[cfg(feature = "packet-codec-msgpack")]
            Self::MsgPack => WsIoPacketMsgPackCodec.decode(bytes),

            #[cfg(feature = "packet-codec-serde-json")]
            Self::SerdeJson => WsIoPacketSerdeJsonCodec.decode(bytes),

            #[cfg(feature = "packet-codec-sonic-rs")]
            Self::SonicRs => WsIoPacketSonicRsCodec.decode(bytes),
        }
    }
}
