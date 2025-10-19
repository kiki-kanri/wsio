use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;
use url::Url;

use crate::{
    connection::WsIoClientConnection,
    core::packet::codecs::WsIoPacketCodec,
};

pub(crate) struct WsIoClientConfig {
    pub(crate) auth: Option<
        Arc<
            dyn Fn(Arc<WsIoClientConnection>) -> Pin<Box<dyn Future<Output = Result<Option<Vec<u8>>>> + Send + 'static>>
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub(crate) connect_url: Url,
    pub(crate) packet_codec: WsIoPacketCodec,
}
