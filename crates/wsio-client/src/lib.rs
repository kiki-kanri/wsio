use std::{
    fmt::Display,
    sync::Arc,
};

use anyhow::Result;
use serde::Serialize;
use url::Url;
pub use wsio_core as core;

mod builder;
mod config;
pub mod connection;
mod runtime;

use crate::{
    builder::WsIoClientBuilder,
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

    pub async fn emit<D: Serialize>(&self, event: impl Into<String>, data: Option<&D>) -> Result<()> {
        self.0.emit(event, data).await
    }

    pub async fn disconnect(&self) {
        self.0.disconnect().await
    }
}
