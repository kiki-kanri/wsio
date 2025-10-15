use serde::{
    Deserialize,
    Serialize,
};

pub mod codecs;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WsIoPacketType {
    #[serde(rename = "0")]
    Auth,

    #[serde(rename = "1")]
    Event,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WsIoPacket<D> {
    #[serde(rename = "k")]
    key: String,

    #[serde(rename = "d")]
    data: D,

    #[serde(rename = "t")]
    packet_type: WsIoPacketType,
}
