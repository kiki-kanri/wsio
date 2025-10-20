use std::sync::{
    Arc,
    LazyLock,
};

use anyhow::Result;
use axum::{
    Router,
    serve,
};
use kikiutils::{
    signal::wait_for_shutdown_signal,
    task::manager::TaskManager,
};
use tikv_jemallocator::Jemalloc;
use tokio::net::TcpListener;
use wsio_server::{
    WsIoServer,
    connection::WsIoServerConnection,
    core::packet::codecs::WsIoPacketCodec,
};

// Constants
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
static MAIN_TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(TaskManager::new);
static WS_IO_SERVER: LazyLock<WsIoServer> = LazyLock::new(|| WsIoServer::builder().build());

// Functions
async fn on_connect(connection: Arc<WsIoServerConnection>) -> Result<()> {
    println!(
        "on_connect, sid: {}, ns conns: {}, server conns: {}",
        connection.sid(),
        connection.namespace().connection_count(),
        connection.namespace().server().connection_count()
    );

    Ok(())
}

async fn on_ready(connection: Arc<WsIoServerConnection>) -> Result<()> {
    println!(
        "on_ready, sid: {}, ns conns: {}, server conns: {}",
        connection.sid(),
        connection.namespace().connection_count(),
        connection.namespace().server().connection_count()
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("pid: {}", std::process::id());

    // Register /auth namespace
    WS_IO_SERVER
        .new_namespace_builder("/auth")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::Bincode)
        .with_auth(|_, _: Option<&()>| async { Ok(()) })
        .register()?;

    // Register /bincode namespace
    WS_IO_SERVER
        .new_namespace_builder("/bincode")?
        .on_connect(on_connect)
        .on_ready(on_ready)
        .packet_codec(WsIoPacketCodec::Bincode)
        .register()?;

    // Register /msg-pack namespace
    WS_IO_SERVER
        .new_namespace_builder("/msg-pack")?
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

    println!("Namespace count: {}", WS_IO_SERVER.namespace_count());
    let ws_io_layer = WS_IO_SERVER.layer();
    let app = Router::new().layer(ws_io_layer);
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    println!("Listening on {}", listener.local_addr().unwrap());
    MAIN_TASK_MANAGER.spawn_with_token(async move |token| {
        println!("Started");
        let _ = serve(listener, app)
            .with_graceful_shutdown(async move { token.cancelled().await })
            .await;
    });

    let _ = wait_for_shutdown_signal().await;
    MAIN_TASK_MANAGER.cancel_and_join_existing().await;
    println!("Stopped");
    Ok(())
}
