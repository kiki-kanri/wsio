use std::{
    fmt::Debug as FmtDebug,
    task::{
        Context,
        Poll,
    },
};

use http::{
    Request,
    Response,
};
use http_body::Body;
use tower_service::Service;

use crate::runtime::WsIoRuntime;

#[derive(Clone, Debug)]
pub struct WsIoService<S> {
    inner: S,
    runtime: WsIoRuntime,
}

impl<S> WsIoService<S> {
    pub fn new(inner: S, runtime: WsIoRuntime) -> Self {
        Self { inner, runtime }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for WsIoService<S>
where
    ReqBody: Body + Send + Unpin + FmtDebug + 'static,
    ReqBody::Data: Send,
    ReqBody::Error: FmtDebug,
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone,
{
    type Error = S::Error;
    type Future = S::Future;
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;
    type Response = Response<ResBody>;

    #[inline(always)]
    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        if request
            .uri()
            .path()
            .starts_with(self.runtime.config.request_path.as_ref())
        {
            println!("Intercepted ws.io request");
        }

        self.inner.call(request)
    }

    #[inline(always)]
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(ctx)
    }
}
