use std::sync::Arc;

use futures::StreamExt;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Role;

use crate::runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIoNamespace {
    path: String,
    runtime: Arc<WsIoRuntime>,
}

impl WsIoNamespace {
    pub(crate) fn new(path: &str, runtime: Arc<WsIoRuntime>) -> WsIoNamespace {
        WsIoNamespace {
            path: path.to_string(),
            runtime,
        }
    }

    pub(crate) async fn handle_upgraded_ws(&self, upgraded_ws: Upgraded) {
        let io = TokioIo::new(upgraded_ws);
        let ws_stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        let (tx, rx) = ws_stream.split();
    }
}
