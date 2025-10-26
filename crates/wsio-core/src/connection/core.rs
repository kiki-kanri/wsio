use anyhow::Result;
use tokio::{
    select,
    spawn,
    sync::mpsc::{
        Receiver,
        Sender,
        channel,
    },
};
use tokio_util::sync::CancellationToken;
use tungstenite::{
    Message,
    protocol::WebSocketConfig,
};

use crate::{
    atomic::status::AtomicStatus,
    channel_capacity_from_websocket_config,
};

pub struct WsIoConnectionCore<S: Into<u8> + TryFrom<u8>> {
    pub cancel_token: CancellationToken,
    pub message_tx: Sender<Message>,
    pub status: AtomicStatus<S>,
}

impl<S: Into<u8> + TryFrom<u8>> WsIoConnectionCore<S> {
    #[inline]
    pub fn new(status: S, websocket_config: &WebSocketConfig) -> (Self, Receiver<Message>) {
        let channel_capacity = channel_capacity_from_websocket_config(websocket_config);
        let (message_tx, message_rx) = channel(channel_capacity);
        (
            Self {
                cancel_token: CancellationToken::new(),
                message_tx,
                status: AtomicStatus::new(status),
            },
            message_rx,
        )
    }

    // Public methods

    #[inline]
    pub fn spawn_task<F: Future<Output = Result<()>> + Send + 'static>(&self, future: F) {
        let cancel_token = self.cancel_token.clone();
        spawn(async move {
            select! {
                _ = cancel_token.cancelled() => {},
                _ = future => {},
            }
        });
    }
}
