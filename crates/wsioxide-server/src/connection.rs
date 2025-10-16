use std::sync::Arc;

use bson::oid::ObjectId;
use http::HeaderMap;
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
    unbounded_channel,
};
use tokio_tungstenite::tungstenite::Message;

use crate::namespace::WsIoServerNamespace;

pub struct WsIoServerConnection {
    authorized: bool,
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    sid: String,
    tx: UnboundedSender<Message>,
}

impl WsIoServerConnection {
    pub(crate) fn new(headers: HeaderMap, namespace: Arc<WsIoServerNamespace>) -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                authorized: false,
                headers,
                namespace,
                sid: ObjectId::new().to_string(),
                tx,
            },
            rx,
        )
    }

    // Protected methods
    pub(crate) async fn init(&self) {}

    pub(crate) fn on_close(&self) {}

    pub(crate) async fn on_message(&self, message: Message) {}

    // Public methods

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
