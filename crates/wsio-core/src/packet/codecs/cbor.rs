use std::{
    io::Cursor,
    sync::LazyLock,
};

use anyhow::Result;
use ciborium::{
    de::from_reader,
    ser::into_writer,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::super::WsIoPacket;

pub(super) struct WsIoPacketCborCodec;

static EMPTY_DATA_ENCODED: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut buffer = Vec::new();
    into_writer(&(), &mut buffer).unwrap();
    buffer
});

impl WsIoPacketCborCodec {
    pub(super) const IS_TEXT: bool = false;

    #[inline]
    pub(super) fn decode(&self, bytes: &[u8]) -> Result<WsIoPacket> {
        Ok(from_reader(Cursor::new(bytes))?)
    }

    #[inline]
    pub(super) fn decode_data<D: DeserializeOwned>(&self, bytes: &[u8]) -> Result<D> {
        Ok(from_reader(Cursor::new(bytes))?)
    }

    #[inline]
    pub(super) fn encode(&self, packet: &WsIoPacket) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        into_writer(packet, &mut buffer)?;
        Ok(buffer)
    }

    #[inline]
    pub(super) fn empty_data_encoded(&self) -> &[u8] {
        EMPTY_DATA_ENCODED.as_ref()
    }

    #[inline]
    pub(super) fn encode_data<D: Serialize>(&self, data: &D) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        into_writer(data, &mut buffer)?;
        Ok(buffer)
    }
}
