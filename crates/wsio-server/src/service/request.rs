use std::sync::Arc;

use http::{
    Method,
    Request,
    Response,
    StatusCode,
    header,
};
use url::form_urlencoded;

use crate::runtime::WsIoRuntime;

pub async fn dispatch_request<ReqBody, ResBody: Default, E: Send>(
    request: Request<ReqBody>,
    runtime: Arc<WsIoRuntime>,
) -> Result<Response<ResBody>, E> {
    if request.method() != Method::GET {
        return respond(StatusCode::METHOD_NOT_ALLOWED);
    }

    if !request
        .headers()
        .get(header::UPGRADE)
        .and_then(|h| h.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
    {
        return respond(StatusCode::BAD_REQUEST);
    }

    let Some(namespace_path) = request.uri().query().and_then(|q| {
        form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "namespace")
            .map(|(_, v)| v.into_owned())
    }) else {
        return respond(StatusCode::BAD_REQUEST);
    };

    let Some(namespace) = runtime.get_namespace(&namespace_path) else {
        return respond(StatusCode::NOT_FOUND);
    };

    respond(StatusCode::SWITCHING_PROTOCOLS)
}

#[inline]
fn respond<ResBody: Default, E: Send>(status: StatusCode) -> Result<Response<ResBody>, E> {
    Ok(Response::builder().status(status).body(ResBody::default()).unwrap())
}
