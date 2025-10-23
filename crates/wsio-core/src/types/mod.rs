use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

type AsyncUnaryResultHandler<P> =
    dyn Fn(Arc<P>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>> + Send + Sync + 'static;

pub type ArcAsyncUnaryResultHandler<P> = Arc<AsyncUnaryResultHandler<P>>;
pub type BoxAsyncUnaryResultHandler<P> = Box<AsyncUnaryResultHandler<P>>;
