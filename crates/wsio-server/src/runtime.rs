use std::sync::{
    Arc,
    Weak,
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

use crate::{
    config::WsIoServerConfig,
    connection::WsIoServerConnection,
    core::atomic::status::AtomicStatus,
    namespace::{
        WsIoServerNamespace,
        builder::WsIoServerNamespaceBuilder,
    },
};

#[repr(u8)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
enum RuntimeStatus {
    Running,
    Stopped,
    Stopping,
}

pub(crate) struct WsIoServerRuntime {
    pub(crate) config: WsIoServerConfig,
    connections: DashMap<String, Weak<WsIoServerConnection>>,
    namespaces: DashMap<String, Arc<WsIoServerNamespace>>,
    status: AtomicStatus<RuntimeStatus>,
}

impl WsIoServerRuntime {
    pub(crate) fn new(config: WsIoServerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            connections: DashMap::new(),
            namespaces: DashMap::new(),
            status: AtomicStatus::new(RuntimeStatus::Running),
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
    pub(crate) fn insert_connection(&self, connection: &Arc<WsIoServerConnection>) {
        self.connections
            .insert(connection.sid().into(), Arc::downgrade(connection));
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

    pub(crate) async fn remove_namespace(&self, path: &str) {
        let Some((_, namespace)) = self.namespaces.remove(path) else {
            return;
        };

        namespace.shutdown().await;
    }

    pub(crate) async fn shutdown(&self) {
        match self.status.get() {
            RuntimeStatus::Stopped => return,
            RuntimeStatus::Running => self.status.store(RuntimeStatus::Stopping),
            _ => unreachable!(),
        }

        join_all(self.namespaces.iter().map(|entry| {
            let namespace = entry.value().clone();
            async move { namespace.shutdown().await }
        }))
        .await;

        self.status.store(RuntimeStatus::Stopped);
    }
}
