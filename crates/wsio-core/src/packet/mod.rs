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

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum WsIoPacketType {
    Auth = 0,
    Disconnect = 1,
    Event = 2,
    Init = 3,
    Ready = 4,
}

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
