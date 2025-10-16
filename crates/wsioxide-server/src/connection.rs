use std::sync::Arc;

use anyhow::Result;
use bson::oid::ObjectId;
use http::HeaderMap;
use serde::Serialize;
use tokio::sync::{
    RwLock,
    mpsc::{
        UnboundedReceiver,
        UnboundedSender,
        unbounded_channel,
    },
};
use tokio_tungstenite::tungstenite::Message;
use wsioxide_core::packet::{
    WsIoPacket,
    WsIoPacketType,
};

use crate::namespace::WsIoServerNamespace;

pub enum WsIoConnectionStatus {
    AwaitingAuth,
    Created,
    Ready,
}

pub struct WsIoServerConnection {
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    sid: String,
    status: RwLock<WsIoConnectionStatus>,
    tx: UnboundedSender<Message>,
}

impl WsIoServerConnection {
    pub(crate) fn new(headers: HeaderMap, namespace: Arc<WsIoServerNamespace>) -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                headers,
                namespace,
                sid: ObjectId::new().to_string(),
                status: RwLock::new(WsIoConnectionStatus::AwaitingAuth),
                tx,
            },
            rx,
        )
    }

    // Protected methods
    pub(crate) async fn activate(self: &Arc<Self>) -> Result<()> {
        self.namespace.add_connection(self.clone());
        *self.status.write().await = WsIoConnectionStatus::Ready;
        Ok(())
    }

    pub(crate) fn cleanup(&self) {
        self.namespace.cleanup_connection(&self.sid);
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        #[derive(Serialize)]
        struct Data(String, bool);

        let require_auth = self.namespace.requires_auth();
        let packet = WsIoPacket {
            data: Some(Data(self.sid.clone(), require_auth)),
            key: None,
            r#type: WsIoPacketType::Init,
        };

        self.send_packet(packet)?;
        if require_auth {
            *self.status.write().await = WsIoConnectionStatus::AwaitingAuth
        } else {
            self.activate().await?;
        }

        Ok(())
    }

    pub(crate) async fn on_message(&self, message: Message) {}

    #[inline]
    pub(crate) fn send(&self, message: Message) {
        let _ = self.tx.send(message);
    }

    #[inline]
    pub(crate) fn send_packet<D: Serialize>(&self, packet: WsIoPacket<D>) -> Result<()> {
        self.send(self.namespace.encode_packet_to_message(packet)?);
        Ok(())
    }

    // Public methods

    #[inline]
    pub fn close(&self) {
        let _ = self.tx.send(Message::Close(None));
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
