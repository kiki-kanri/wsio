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

use crate::{
    core::packet::{
        WsIoPacket,
        WsIoPacketType,
    },
    namespace::WsIoServerNamespace,
};

enum WsIoServerConnectionStatus {
    Activating,
    AwaitingAuth,
    Closed,
    Closing,
    Created,
    Ready,
}

pub struct WsIoServerConnection {
    headers: HeaderMap,
    namespace: Arc<WsIoServerNamespace>,
    sid: String,
    status: RwLock<WsIoServerConnectionStatus>,
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
                status: RwLock::new(WsIoServerConnectionStatus::Created),
                tx,
            },
            rx,
        )
    }

    // Protected methods
    pub(crate) async fn activate(self: &Arc<Self>) -> Result<()> {
        *self.status.write().await = WsIoServerConnectionStatus::Activating;
        // TODO: middlewares
        self.namespace.insert_connection(self.clone());
        self.namespace.on_connect(self.clone()).await?;
        *self.status.write().await = WsIoServerConnectionStatus::Ready;
        Ok(())
    }

    pub(crate) async fn cleanup(&self) {
        *self.status.write().await = WsIoServerConnectionStatus::Closing;
        self.namespace.cleanup_connection(&self.sid);
        *self.status.write().await = WsIoServerConnectionStatus::Closed;
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
            *self.status.write().await = WsIoServerConnectionStatus::AwaitingAuth
        } else {
            self.activate().await?;
        }

        Ok(())
    }

    pub(crate) async fn on_message(&self, _message: Message) {}

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
    pub fn namespace(&self) -> Arc<WsIoServerNamespace> {
        self.namespace.clone()
    }

    #[inline]
    pub fn sid(&self) -> &str {
        &self.sid
    }
}
