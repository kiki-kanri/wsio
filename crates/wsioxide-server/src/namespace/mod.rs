use std::sync::Arc;

pub(crate) mod builder;
mod config;

use builder::WsIoServerNamespaceBuilder;
use config::WsIoServerNamespaceConfig;
use dashmap::DashMap;
use hyper::upgrade::Upgraded;

use crate::{
    connection::WsIoServerConnection,
    runtime::WsIoServerRuntime,
};

#[derive(Clone)]
pub struct WsIoServerNamespace {
    config: WsIoServerNamespaceConfig,
    connections: DashMap<String, Arc<WsIoServerConnection>>,
    runtime: Arc<WsIoServerRuntime>,
}

impl WsIoServerNamespace {
    fn new(config: WsIoServerNamespaceConfig, runtime: Arc<WsIoServerRuntime>) -> Self {
        Self {
            config,
            connections: DashMap::new(),
            runtime,
        }
    }

    // Protected methods
    pub(crate) fn builder(path: impl AsRef<str>, runtime: Arc<WsIoServerRuntime>) -> WsIoServerNamespaceBuilder {
        WsIoServerNamespaceBuilder::new(path.as_ref(), runtime)
    }

    pub(crate) async fn register_connection(&self, connection: Arc<WsIoServerConnection>, upgraded: Upgraded) {
        self.connections
            .insert(connection.sid().to_string(), connection.clone());
    }

    // Public methods
    pub fn path(&self) -> &str {
        &self.config.path
    }
}
