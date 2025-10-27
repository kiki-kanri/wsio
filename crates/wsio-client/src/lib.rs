use std::{
    fmt::Display,
    sync::Arc,
};

use anyhow::Result;
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use url::Url;
pub use wsio_core as core;

mod builder;
mod config;
pub mod connection;
mod runtime;

use crate::{
    builder::WsIoClientBuilder,
    connection::WsIoClientConnection,
    runtime::WsIoClientRuntime,
};

#[derive(Clone)]
pub struct WsIoClient(Arc<WsIoClientRuntime>);

impl WsIoClient {
    // Public methods
    pub fn builder<U>(url: U) -> Result<WsIoClientBuilder>
    where
        U: TryInto<Url>,
        U::Error: Display,
    {
        let url = match url.try_into() {
            Ok(url) => url,
            Err(e) => panic!("Invalid URL: {e}"),
        };

        WsIoClientBuilder::new(url)
    }

    pub async fn connect(&self) {
        self.0.connect().await
    }

    pub async fn disconnect(&self) {
        self.0.disconnect().await
    }

    pub async fn emit<D: Serialize>(&self, event: impl Into<String>, data: Option<&D>) -> Result<()> {
        self.0.emit(event, data).await
    }

    #[inline]
    pub fn off(&self, event: impl AsRef<str>) {
        self.0.off(event);
    }

    #[inline]
    pub fn off_by_handler_id(&self, event: impl AsRef<str>, handler_id: u32) {
        self.0.off_by_handler_id(event, handler_id);
    }

    #[inline]
    pub fn on<H, Fut, D>(&self, event: impl Into<String>, handler: H) -> u32
    where
        H: Fn(Arc<WsIoClientConnection>, Arc<D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + Sync + 'static,
    {
        self.0.on(event, handler)
    }
}
