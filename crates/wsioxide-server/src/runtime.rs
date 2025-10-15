use std::sync::Arc;

use anyhow::{
    Result,
    bail,
};
use dashmap::DashMap;

use crate::{
    config::WsIoConfig,
    namespace::{
        WsIoNamespace,
        builder::WsIoNamespaceBuilder,
    },
};

pub(crate) struct WsIoRuntime {
    pub(crate) config: WsIoConfig,
    namespaces: DashMap<String, Arc<WsIoNamespace>>,
}

impl WsIoRuntime {
    pub(crate) fn new(config: WsIoConfig) -> Self {
        WsIoRuntime {
            config,
            namespaces: DashMap::new(),
        }
    }

    // Protected methods

    #[inline]
    pub(crate) fn get_namespace(&self, path: impl AsRef<str>) -> Option<Arc<WsIoNamespace>> {
        self.namespaces.get(path.as_ref()).map(|v| v.clone())
    }

    #[inline]
    pub(crate) fn insert_namespace(&self, namespace: Arc<WsIoNamespace>) -> Result<()> {
        if self.namespaces.contains_key(namespace.path()) {
            bail!("Namespace {} already exists", namespace.path());
        }

        self.namespaces.insert(namespace.path().into(), namespace);
        Ok(())
    }

    #[inline]
    pub(crate) fn new_namespace_builder(self: &Arc<Self>, path: impl AsRef<str>) -> Result<WsIoNamespaceBuilder> {
        let path = path.as_ref();
        if self.namespaces.contains_key(path) {
            bail!("Namespace {} already exists", path);
        }

        Ok(WsIoNamespaceBuilder::new(path, self.clone()))
    }
}
