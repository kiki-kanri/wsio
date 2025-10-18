use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

use crate::connection::WsIoServerConnection;

pub(crate) type WsIoServerNamespaceAuthHandler = Arc<
    dyn for<'a> Fn(Arc<WsIoServerConnection>, Option<&'a [u8]>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) type WsIoServerConnectionOnConnectHandler = Box<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

pub(crate) type WsIoServerConnectionOnDisconnectHandler = Box<
    dyn Fn(Arc<WsIoServerConnection>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;
