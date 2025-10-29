use std::sync::Arc;

use http::{
    HeaderName,
    Method,
    Request,
    Response,
    StatusCode,
    header::{
        CONNECTION,
        SEC_WEBSOCKET_ACCEPT,
        SEC_WEBSOCKET_KEY,
        SEC_WEBSOCKET_VERSION,
        UPGRADE,
    },
};
use hyper::upgrade::OnUpgrade;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;
use url::form_urlencoded;

use crate::runtime::WsIoServerRuntime;

// Functions
#[inline]
fn check_header_value<ReqBody>(request: &Request<ReqBody>, name: HeaderName, expected_value: &[u8]) -> bool {
    match request.headers().get(name) {
        Some(value) => value.as_bytes().eq_ignore_ascii_case(expected_value),
        None => false,
    }
}

pub(super) async fn dispatch_request<ReqBody, ResBody: Default, E: Send>(
    mut request: Request<ReqBody>,
    runtime: Arc<WsIoServerRuntime>,
) -> Result<Response<ResBody>, E> {
    // Check method
    if request.method() != Method::GET {
        return respond(StatusCode::METHOD_NOT_ALLOWED);
    }

    // Check required headers
    if !check_header_value(&request, UPGRADE, b"websocket")
        || !check_header_value(&request, CONNECTION, b"upgrade")
        || !check_header_value(&request, SEC_WEBSOCKET_VERSION, b"13")
    {
        return respond(StatusCode::BAD_REQUEST);
    }

    // Get websocket sec key
    let Some(ws_sec_key) = request.headers().get(SEC_WEBSOCKET_KEY).and_then(|v| v.to_str().ok()) else {
        return respond(StatusCode::BAD_REQUEST);
    };

    // Get namespace path
    let Some((_, namespace_path)) = request
        .uri()
        .query()
        .and_then(|q| form_urlencoded::parse(q.as_bytes()).find(|(k, _)| k == "namespace"))
    else {
        return respond(StatusCode::BAD_REQUEST);
    };

    // Get namespace
    let Some(namespace) = runtime.get_namespace(&namespace_path) else {
        return respond(StatusCode::NOT_FOUND);
    };

    // Generate accept key
    let ws_accept_key = derive_accept_key(ws_sec_key.as_bytes());

    // Upgrade
    let on_upgrade = match request.extensions_mut().remove::<OnUpgrade>() {
        Some(upgrade) => upgrade,
        None => return respond(StatusCode::INTERNAL_SERVER_ERROR),
    };

    namespace
        .handle_on_upgrade_request(request.headers().clone(), on_upgrade)
        .await;

    Ok(Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(CONNECTION, "Upgrade")
        .header(SEC_WEBSOCKET_ACCEPT, ws_accept_key)
        .header(UPGRADE, "websocket")
        .body(ResBody::default())
        .unwrap())
}

#[inline]
fn respond<ResBody: Default, E: Send>(status: StatusCode) -> Result<Response<ResBody>, E> {
    Ok(Response::builder().status(status).body(ResBody::default()).unwrap())
}
