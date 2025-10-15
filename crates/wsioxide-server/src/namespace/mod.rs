use std::sync::Arc;

use futures::StreamExt;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Role;

pub(crate) mod builder;
mod config;

use builder::WsIoServerNamespaceBuilder;
use config::WsIoServerNamespaceConfig;

use crate::runtime::WsIoServerRuntime;

#[derive(Clone)]
pub struct WsIoServerNamespace {
    runtime: Arc<WsIoServerRuntime>,
    config: WsIoServerNamespaceConfig,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Self {
        Self { config, runtime }
    }

    // Protected methods
    pub(crate) fn builder(path: impl AsRef<str>, runtime: Arc<WsIoServerRuntime>) -> WsIoServerNamespaceBuilder {
        WsIoServerNamespaceBuilder::new(path.as_ref(), runtime)
    }

    pub(crate) async fn handle_upgraded_ws(&self, upgraded_ws: Upgraded) {
        let io = TokioIo::new(upgraded_ws);
        let ws_stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
        let (tx, rx) = ws_stream.split();
    }

    // Public methods
    pub fn path(&self) -> &str {
        &self.config.path
    }
}
