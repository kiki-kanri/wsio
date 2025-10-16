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
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WsIoPacket<D> {
    #[serde(rename = "k")]
    key: String,

    #[serde(rename = "d")]
    data: Option<D>,

    #[serde(rename = "t")]
    packet_type: WsIoPacketType,
}
