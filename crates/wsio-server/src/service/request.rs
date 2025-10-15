use std::sync::Arc;

use http::{
    HeaderName,
    Method,
    Request,
    Response,
    StatusCode,
    header,
};
use hyper::upgrade::OnUpgrade;
use tokio::spawn;
use tungstenite::handshake::derive_accept_key;
use url::form_urlencoded;

use crate::runtime::WsIoRuntime;

#[inline]
fn check_header_value<ReqBody>(request: &Request<ReqBody>, name: HeaderName, expected_value: &str) -> bool {
    get_header_value(request, name)
        .map(|v| v.eq_ignore_ascii_case(expected_value))
        .unwrap_or(false)
}

pub async fn dispatch_request<ReqBody, ResBody: Default, E: Send>(
    mut request: Request<ReqBody>,
    runtime: Arc<WsIoRuntime>,
) -> Result<Response<ResBody>, E> {
    // Check method
    if request.method() != Method::GET {
        return respond(StatusCode::METHOD_NOT_ALLOWED);
    }

    // Check upgrade header
    if !check_header_value(&request, header::UPGRADE, "websocket") {
        return respond(StatusCode::BAD_REQUEST);
    }

    // Check connection header
    if !check_header_value(&request, header::CONNECTION, "upgrade") {
        return respond(StatusCode::BAD_REQUEST);
    }

    // Check websocket version header
    if !check_header_value(&request, header::SEC_WEBSOCKET_VERSION, "13") {
        return respond(StatusCode::BAD_REQUEST);
    }

    // Get websocket sec key
    let Some(ws_sec_key) = get_header_value(&request, header::SEC_WEBSOCKET_KEY) else {
        return respond(StatusCode::BAD_REQUEST);
    };

    // Get namespace path
    let Some(namespace_path) = request.uri().query().and_then(|q| {
        form_urlencoded::parse(q.as_bytes())
            .find(|(k, _)| k == "namespace")
            .map(|(_, v)| v.into_owned())
    }) else {
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

    spawn(async move {
        match on_upgrade.await {
            Ok(upgraded) => namespace.handle_upgraded_ws(upgraded).await,
            Err(e) => {}
        }
    });

    Ok(Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(header::CONNECTION, "Upgrade")
        .header(header::SEC_WEBSOCKET_ACCEPT, ws_accept_key)
        .header(header::UPGRADE, "websocket")
        .body(ResBody::default())
        .unwrap())
}

#[inline]
fn get_header_value<ReqBody>(request: &Request<ReqBody>, name: HeaderName) -> Option<&str> {
    request.headers().get(name).and_then(|v| v.to_str().ok())
}

#[inline]
fn respond<ResBody: Default, E: Send>(status: StatusCode) -> Result<Response<ResBody>, E> {
    Ok(Response::builder().status(status).body(ResBody::default()).unwrap())
}
