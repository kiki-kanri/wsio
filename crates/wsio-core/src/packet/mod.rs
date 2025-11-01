use serde::{
    Deserialize,
    Serialize,
};
use serde_repr::{
    Deserialize_repr,
    Serialize_repr,
};
use serde_with::skip_serializing_none;

pub mod codecs;

// Enums
#[repr(u8)]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
pub enum WsIoPacketType {
    Disconnect = 0,
    Event = 1,
    Init = 2,
    Ready = 3,
}

// Structs
#[cfg(any(
    feature = "packet-codec-bincode",
    feature = "packet-codec-msgpack",
    feature = "packet-codec-postcard"
))]
#[derive(Deserialize)]
struct InnerPacket(Option<Vec<u8>>, Option<String>, WsIoPacketType);

#[cfg(any(
    feature = "packet-codec-bincode",
    feature = "packet-codec-msgpack",
    feature = "packet-codec-postcard"
))]
#[derive(Serialize)]
struct InnerPacketRef<'a>(&'a Option<Vec<u8>>, &'a Option<String>, &'a WsIoPacketType);

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WsIoPacket {
    #[serde(rename = "d")]
    pub data: Option<Vec<u8>>,

    #[serde(rename = "k")]
    pub key: Option<String>,

    #[serde(rename = "t")]
    pub r#type: WsIoPacketType,
}

impl WsIoPacket {
    #[inline]
    pub fn new(r#type: WsIoPacketType, key: Option<&str>, data: Option<Vec<u8>>) -> Self {
        Self {
            data,
            key: key.map(|k| k.into()),
            r#type,
        }
    }

    // Public methods
    #[inline]
    pub fn new_disconnect() -> Self {
        Self::new(WsIoPacketType::Disconnect, None, None)
    }

    #[inline]
    pub fn new_event(event: &str, data: Option<Vec<u8>>) -> Self {
        Self::new(WsIoPacketType::Event, Some(event), data)
    }

    #[inline]
    pub fn new_init(data: Option<Vec<u8>>) -> Self {
        Self::new(WsIoPacketType::Init, None, data)
    }

    #[inline]
    pub fn new_ready() -> Self {
        Self::new(WsIoPacketType::Ready, None, None)
    }
}
