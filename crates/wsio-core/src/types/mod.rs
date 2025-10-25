use std::{
    pin::Pin,
    sync::Arc,
};

use anyhow::Result;

type AsyncUnaryResultHandler<P, R = ()> =
    dyn Fn(Arc<P>) -> Pin<Box<dyn Future<Output = Result<R>> + Send + 'static>> + Send + Sync + 'static;

pub type ArcAsyncUnaryResultHandler<P, R = ()> = Arc<AsyncUnaryResultHandler<P, R>>;
pub type BoxAsyncUnaryResultHandler<P, R = ()> = Box<AsyncUnaryResultHandler<P, R>>;
