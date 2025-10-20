use std::sync::Arc;

use futures::{
    SinkExt,
    StreamExt,
};
use http::{
    HeaderName,
    Method,
    Request,
    Response,
    StatusCode,
    header,
};
use hyper::upgrade::OnUpgrade;
use hyper_util::rt::TokioIo;
use tokio::{
    join,
    select,
    spawn,
};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::protocol::Role,
};
use tungstenite::{
    Message,
    handshake::derive_accept_key,
};
use url::form_urlencoded;

use crate::{
    connection::WsIoServerConnection,
    runtime::WsIoServerRuntime,
};

#[inline]
fn check_header_value<ReqBody>(request: &Request<ReqBody>, name: HeaderName, expected_value: &str) -> bool {
    get_header_value(request, name)
        .map(|v| v.eq_ignore_ascii_case(expected_value))
        .unwrap_or(false)
}

pub(super) async fn dispatch_request<ReqBody, ResBody: Default, E: Send>(
    mut request: Request<ReqBody>,
    runtime: Arc<WsIoServerRuntime>,
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

    let headers = request.headers().clone();
    spawn(async move {
        if let Ok(upgraded) = on_upgrade.await {
            let (connection, mut message_rx) = WsIoServerConnection::new(headers, namespace);
            let connection = Arc::new(connection);

            let ws_stream = WebSocketStream::from_raw_socket(TokioIo::new(upgraded), Role::Server, None).await;
            let (mut ws_stream_writer, mut ws_stream_reader) = ws_stream.split();

            let connection_clone = connection.clone();
            let read_ws_stream_task = spawn(async move {
                while let Some(message) = ws_stream_reader.next().await {
                    if match message {
                        Ok(Message::Binary(bytes)) => connection_clone.handle_incoming_packet(&bytes).await,
                        Ok(Message::Close(_)) => break,
                        Ok(Message::Text(text)) => connection_clone.handle_incoming_packet(text.as_bytes()).await,
                        Err(_) => break,
                        _ => Ok(()),
                    }
                    .is_err()
                    {
                        break;
                    }
                }
            });

            let write_ws_stream_task = spawn(async move {
                while let Some(message) = message_rx.recv().await {
                    let is_close = matches!(message, Message::Close(_));
                    if ws_stream_writer.send(message).await.is_err() {
                        break;
                    }

                    if is_close {
                        let _ = ws_stream_writer.flush().await;
                        break;
                    }
                }
            });

            match connection.init().await {
                Ok(_) => {
                    select! {
                        _ = read_ws_stream_task => {},
                        _ = write_ws_stream_task => {},
                    }
                }
                Err(_) => {
                    connection.close().await;
                    read_ws_stream_task.abort();
                    let _ = join!(read_ws_stream_task, write_ws_stream_task);
                }
            }

            connection.cleanup().await;
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
