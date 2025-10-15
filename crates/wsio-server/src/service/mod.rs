use std::{
    fmt::Debug as FmtDebug,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use http::{
    Error as HttpError,
    Request,
    Response,
};
use http_body::Body;
use tower_service::Service as TowerService;

mod request;

use request::dispatch_request;

use crate::runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIoService<S> {
    inner: S,
    runtime: WsIoRuntime,
}

impl<S> WsIoService<S> {
    pub(crate) fn new(inner: S, runtime: WsIoRuntime) -> Self {
        Self { inner, runtime }
    }
}

impl<S, ReqBody, ResBody> TowerService<Request<ReqBody>> for WsIoService<S>
where
    ReqBody: Body + FmtDebug + Send + Unpin + 'static,
    ReqBody::Data: Send,
    ReqBody::Error: FmtDebug,
    ResBody: Default + Send + 'static,
    S: TowerService<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: From<HttpError> + Send + 'static,
    S::Future: Send + 'static,
{
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Response = Response<ResBody>;

    #[inline(always)]
    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let runtime = self.runtime.clone();
        if request.uri().path().starts_with(&runtime.config.request_path) {
            Box::pin(async move { dispatch_request(request, runtime).await })
        } else {
            Box::pin(async move { inner.call(request).await })
        }
    }

    #[inline(always)]
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }
}
