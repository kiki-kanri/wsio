use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use dashmap::DashMap;

use crate::{
    config::WsIoServerConfig,
    namespace::{
        WsIoServerNamespace,
        builder::WsIoServerNamespaceBuilder,
    },
};

pub(crate) struct WsIoServerRuntime {
    pub(crate) config: WsIoServerConfig,
    namespaces: DashMap<String, Arc<WsIoServerNamespace>>,
}

impl WsIoServerRuntime {
    pub(crate) fn new(config: WsIoServerConfig) -> Self {
        Self {
            config,
            namespaces: DashMap::new(),
        }
    }

    // Protected methods

    #[inline]
    pub(crate) fn get_namespace(&self, path: &str) -> Option<Arc<WsIoServerNamespace>> {
        self.namespaces.get(path).map(|v| v.clone())
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
