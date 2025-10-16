use std::sync::Arc;

pub(crate) mod builder;
mod config;

use config::WsIoServerNamespaceConfig;
use dashmap::DashMap;

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

    // Public methods

    #[inline]
    pub fn path(&self) -> &str {
        &self.config.path
    }
}
