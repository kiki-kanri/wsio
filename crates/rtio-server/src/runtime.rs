use anyhow::{
    Result,
    bail,
};
use dashmap::DashMap;

use crate::{
    config::WsIoConfig,
    namespace::WsIoNamespace,
};

#[derive(Clone, Debug)]
pub(crate) struct WsIoRuntime {
    pub(crate) config: WsIoConfig,
    namespaces: DashMap<String, WsIoNamespace>,
}

impl WsIoRuntime {
    pub(crate) fn new(config: WsIoConfig) -> Self {
        WsIoRuntime {
            config,
            namespaces: DashMap::new(),
        }
    }

    pub(crate) fn add_ns(&self, path: impl AsRef<str>) -> Result<WsIoNamespace> {
        let path = path.as_ref();
        if self.namespaces.contains_key(path) {
            bail!("Namespace {} already exists", path);
        }

        let namespace = WsIoNamespace::new(path.as_ref(), self.clone());
        self.namespaces.insert(path.to_string(), namespace.clone());
        Ok(namespace)
    }

    pub fn of(&self, path: impl AsRef<str>) -> Option<WsIoNamespace> {
        self.namespaces.get(path.as_ref()).map(|v| v.clone())
    }
}
