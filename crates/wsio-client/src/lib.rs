use std::{
    fmt::Display,
    sync::Arc,
};

use anyhow::Result;
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

pub struct WsIoClient(Arc<WsIoClientRuntime>);

impl WsIoClient {
    // Public methods
    pub fn builder<U>(namespace_url: U) -> Result<WsIoClientBuilder>
    where
        U: TryInto<Url>,
        U::Error: Display,
    {
        let namespace_url = match namespace_url.try_into() {
            Ok(namespace_url) => namespace_url,
            Err(e) => panic!("Invalid namespace URL: {}", e),
        };

        WsIoClientBuilder::new(namespace_url)
    }
}
