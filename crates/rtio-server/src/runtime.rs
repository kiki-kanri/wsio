use std::borrow::Cow;

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

    pub(crate) fn add_ns(&self, path: Cow<'static, str>) -> Result<WsIoNamespace> {
        if self.namespaces.contains_key(path.as_ref()) {
            bail!("Namespace {} already exists", path);
        }

        let namespace = WsIoNamespace::new(path.as_ref(), self.clone());
        self.namespaces.insert(path.to_string(), namespace.clone());
        Ok(namespace)
    }
}
