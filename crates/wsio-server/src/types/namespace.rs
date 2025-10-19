use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

use crate::connection::WsIoServerConnection;

pub(crate) type WsIoServerNamespaceMiddleware = Box<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
