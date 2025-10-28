use std::{
    collections::HashMap,
    sync::{
        Arc,
        Weak,
    },
};

use anyhow::{
    Result,
    bail,
};
use dashmap::DashMap;
use futures_util::future::join_all;
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use parking_lot::RwLock;
use serde::Serialize;

use crate::{
    config::WsIoServerConfig,
    connection::WsIoServerConnection,
    core::atomic::status::AtomicStatus,
    namespace::{
        WsIoServerNamespace,
        builder::WsIoServerNamespaceBuilder,
    },
};

// Enums
#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
pub(crate) enum WsIoServerRuntimeStatus {
    Running,
    Stopped,
    Stopping,
}

// Structs
pub(crate) struct WsIoServerRuntime {
    pub(crate) config: WsIoServerConfig,
    connections: DashMap<u64, Weak<WsIoServerConnection>>,
    namespaces: RwLock<HashMap<String, Arc<WsIoServerNamespace>>>,
    pub(crate) status: AtomicStatus<WsIoServerRuntimeStatus>,
}

impl WsIoServerRuntime {
    pub(crate) fn new(config: WsIoServerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            namespaces: RwLock::new(HashMap::new()),
            status: AtomicStatus::new(WsIoServerRuntimeStatus::Running),
        })
    }

    // Private methods
    #[inline]
    fn clone_namespaces(&self) -> Vec<Arc<WsIoServerNamespace>> {
        self.namespaces.read().values().cloned().collect()
    }

    // Protected methods
    #[inline]
    pub(crate) fn connection_count(&self) -> usize {
        self.connections.len()
    }

    pub(crate) async fn emit<D: Serialize>(&self, event: &str, data: Option<&D>) -> Result<()> {
        self.status.ensure(WsIoServerRuntimeStatus::Running, |status| {
            format!("Cannot emit event in invalid status: {:#?}", status)
        })?;

        join_all(
            self.clone_namespaces()
                .iter()
                .map(|namespace| namespace.emit(event, data)),
        )
        .await;

        Ok(())
    }

    #[inline]
    pub(crate) fn get_namespace(&self, path: &str) -> Option<Arc<WsIoServerNamespace>> {
        self.namespaces.read().get(path).cloned()
    }

    #[inline]
    pub(crate) fn insert_connection(&self, connection: &Arc<WsIoServerConnection>) {
        self.connections.insert(connection.id(), Arc::downgrade(connection));
    }

    #[inline]
    pub(crate) fn insert_namespace(&self, namespace: Arc<WsIoServerNamespace>) -> Result<()> {
        if self.namespaces.read().contains_key(namespace.path()) {
            bail!("Namespace {} already exists", namespace.path());
        }

        self.namespaces.write().insert(namespace.path().into(), namespace);
        Ok(())
    }

    #[inline]
    pub(crate) fn namespace_count(&self) -> usize {
        self.namespaces.read().len()
    }

    #[inline]
    pub(crate) fn new_namespace_builder(self: &Arc<Self>, path: &str) -> Result<WsIoServerNamespaceBuilder> {
        if self.namespaces.read().contains_key(path) {
            bail!("Namespace {path} already exists");
        }

        Ok(WsIoServerNamespaceBuilder::new(path, self.clone()))
    }

    #[inline]
    pub(crate) fn remove_connection(&self, id: u64) {
        self.connections.remove(&id);
    }

    pub(crate) async fn remove_namespace(&self, path: &str) {
        let Some(namespace) = self.namespaces.write().remove(path) else {
            return;
        };

        namespace.shutdown().await;
    }

    pub(crate) async fn shutdown(&self) {
        match self.status.get() {
            WsIoServerRuntimeStatus::Stopped => return,
            WsIoServerRuntimeStatus::Running => self.status.store(WsIoServerRuntimeStatus::Stopping),
            _ => unreachable!(),
        }

        join_all(
            self.clone_namespaces()
                .iter()
                .map(|namespace| async move { namespace.shutdown().await }),
        )
        .await;

        self.status.store(WsIoServerRuntimeStatus::Stopped);
    }
}
