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
    pub(crate) fn new(config: WsIoServerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            namespaces: DashMap::new(),
        })
    }

    // Protected methods

    #[inline]
    pub(crate) fn connection_count(&self) -> usize {
        self.connections.len()
    }

    #[inline]
    pub(crate) fn get_namespace(&self, path: &str) -> Option<Arc<WsIoServerNamespace>> {
        self.namespaces.get(path).map(|entry| entry.clone())
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
    pub(crate) fn namespace_count(&self) -> usize {
        self.namespaces.len()
    }

    #[inline]
    pub(crate) fn new_namespace_builder(self: &Arc<Self>, path: &str) -> Result<WsIoServerNamespaceBuilder> {
        if self.namespaces.contains_key(path) {
            bail!("Namespace {path} already exists");
        }

        Ok(WsIoServerNamespaceBuilder::new(path, self.clone()))
    }

    #[inline]
    pub(crate) fn remove_connection(&self, sid: &str) {
        self.connections.remove(sid);
    }
}
