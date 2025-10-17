use serde::{
    Deserialize,
    Serialize,
};
use serde_with::skip_serializing_none;

pub mod codecs;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WsIoPacketType {
    #[serde(rename = "0")]
    Auth,

    #[serde(rename = "1")]
    Event,

    #[serde(rename = "2")]
    Init,
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
