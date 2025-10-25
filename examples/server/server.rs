use std::{
    process::id,
    sync::{
        Arc,
        LazyLock,
    },
};

use anyhow::Result;
use axum::{
    Router,
    serve,
};
use kikiutils::{
    signal::wait_for_shutdown_signal,
    task::manager::TaskManager,
    tracing::init_tracing_with_local_time_format,
};
use tikv_jemallocator::Jemalloc;
use tokio::net::TcpListener;
use wsio_server::{
    WsIoServer,
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
};

// Constants/Statics
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
static MAIN_TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(TaskManager::new);
static WS_IO_SERVER: LazyLock<WsIoServer> = LazyLock::new(|| WsIoServer::builder().build());

// Functions
async fn on_close(connection: Arc<WsIoServerConnection>) -> Result<()> {
    tracing::info!(
        "{}: on_close, sid: {}, ns conns: {}, server conns: {}",
        connection.namespace().path(),
        connection.sid(),
        connection.namespace().connection_count(),
        connection.namespace().server().connection_count()
    );

    Ok(())
}

async fn on_connect(connection: Arc<WsIoServerConnection>) -> Result<()> {
    tracing::info!(
        "{}: on_connect, sid: {}, ns conns: {}, server conns: {}",
        connection.namespace().path(),
        connection.sid(),
        connection.namespace().connection_count(),
        connection.namespace().server().connection_count()
    );

    connection.on_close(on_close).await;
    Ok(())
}

async fn on_ready(connection: Arc<WsIoServerConnection>) -> Result<()> {
    tracing::info!(
        "{}: on_ready, sid: {}, ns conns: {}, server conns: {}",
        connection.namespace().path(),
        connection.sid(),
        connection.namespace().connection_count(),
        connection.namespace().server().connection_count()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = init_tracing_with_local_time_format();
    tracing::info!("pid: {}", id());

    // Register /auth namespace
    WS_IO_SERVER
        .new_namespace_builder("/auth")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .with_auth(|_, _: &()| async { Ok(()) })
        .register()?;

    // Register /bincode namespace
    WS_IO_SERVER
        .new_namespace_builder("/bincode")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::Bincode)
        .register()?;

    // Register /cbor namespace
    WS_IO_SERVER
        .new_namespace_builder("/cbor")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::Cbor)
        .register()?;

    // Register /disconnect namespace
    WS_IO_SERVER
        .new_namespace_builder("/disconnect")?
        .on_ready(|connection| async move {
            connection.disconnect().await;
            Ok(())
        })
        .register()?;

    // Register /msgpack namespace
    WS_IO_SERVER
        .new_namespace_builder("/msgpack")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::MsgPack)
        .register()?;

    // Register /serde-json namespace
    WS_IO_SERVER
        .new_namespace_builder("/serde-json")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::SerdeJson)
        .register()?;

    // Register /sonic-rs namespace
    WS_IO_SERVER
        .new_namespace_builder("/sonic-rs")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::SonicRs)
        .register()?;

    tracing::info!("Namespace count: {}", WS_IO_SERVER.namespace_count());
    let ws_io_layer = WS_IO_SERVER.layer();
    let app = Router::new().layer(ws_io_layer);
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    MAIN_TASK_MANAGER.spawn_with_token(async move |token| {
        tracing::info!("Started");
        let _ = serve(listener, app)
            .with_graceful_shutdown(async move { token.cancelled().await })
            .await;
    });

    let _ = wait_for_shutdown_signal().await;
    MAIN_TASK_MANAGER.cancel_and_join_existing().await;
    tracing::info!("Stopped");
    Ok(())
}
