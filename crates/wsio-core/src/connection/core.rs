use anyhow::Result;
use tokio::{
    select,
    spawn,
    sync::mpsc::Sender,
};
use tokio_util::sync::CancellationToken;
use tungstenite::Message;

use crate::atomic::status::AtomicStatus;

pub struct WsIoConnectionCore<S: Into<u8> + TryFrom<u8>> {
    pub cancel_token: CancellationToken,
    pub message_tx: Sender<Message>,
    pub status: AtomicStatus<S>,
}

impl<S: Into<u8> + TryFrom<u8>> WsIoConnectionCore<S> {
    #[inline]
    pub fn new(message_tx: Sender<Message>, status: S) -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            message_tx,
            status: AtomicStatus::new(status),
        }
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
