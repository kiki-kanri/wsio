use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use dashmap::DashMap;

use crate::{
    config::WsIoServerConfig,
    connection::WsIoServerConnection,
    namespace::{
        WsIoServerNamespace,
        builder::WsIoServerNamespaceBuilder,
    },
};

pub(crate) struct WsIoServerRuntime {
    pub(crate) config: WsIoServerConfig,
    connections: DashMap<String, Arc<WsIoServerConnection>>,
    namespaces: DashMap<String, Arc<WsIoServerNamespace>>,
}

impl WsIoServerRuntime {
    pub(crate) fn new(config: WsIoServerConfig) -> Self {
        Self {
            config,
            connections: DashMap::new(),
            namespaces: DashMap::new(),
        }
    }

    // Protected methods

    #[inline]
    pub(crate) fn cleanup_connection(&self, sid: &str) {
        self.connections.remove(sid);
    }

    #[inline]
    pub(crate) fn connection_count(&self) -> usize {
        self.connections.len()
    }

    #[inline]
    pub(crate) fn get_namespace(&self, path: &str) -> Option<Arc<WsIoServerNamespace>> {
        self.namespaces.get(path).map(|v| v.clone())
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: Arc<WsIoServerConnection>) {
        self.connections.insert(connection.sid().into(), connection);
    }

    #[inline]
    pub(crate) fn insert_namespace(&self, namespace: Arc<WsIoServerNamespace>) -> Result<()> {
        if self.namespaces.contains_key(namespace.path()) {
            bail!("Namespace {} already exists", namespace.path());
        }

        self.namespaces.insert(namespace.path().into(), namespace);
        Ok(())
    }

    #[inline]
    pub(crate) fn new_namespace_builder(self: &Arc<Self>, path: &str) -> Result<WsIoServerNamespaceBuilder> {
        if self.namespaces.contains_key(path) {
            bail!("Namespace {} already exists", path);
        }

        Ok(WsIoServerNamespaceBuilder::new(path, self.clone()))
    }
}
