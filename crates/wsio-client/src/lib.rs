use std::sync::Arc;

pub use wsio_core as core;

mod builder;
mod config;
mod runtime;

use crate::{
    builder::WsIoClientBuilder,
    runtime::WsIoClientRuntime,
};

pub struct WsIoClient(Arc<WsIoClientRuntime>);

impl WsIoClient {
    // Public methods
    pub fn builder() -> WsIoClientBuilder {
        WsIoClientBuilder::new()
    }
}
