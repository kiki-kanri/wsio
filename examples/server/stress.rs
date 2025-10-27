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
    core::packet::codecs::WsIoPacketCodec,
    namespace::WsIoServerNamespace,
};

// Constants/Statics
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
static MAIN_TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(TaskManager::new);
static WS_IO_SERVER: LazyLock<WsIoServer> = LazyLock::new(|| WsIoServer::builder().build());

// Namespaces
static POSTCARD: LazyLock<Arc<WsIoServerNamespace>> = LazyLock::new(|| {
    WS_IO_SERVER
        .new_namespace_builder("/postcard")
        .unwrap()
        .on_connect(|_| async { Ok(()) })
        .packet_codec(WsIoPacketCodec::Postcard)
        .register()
        .unwrap()
});

static SONIC_RS: LazyLock<Arc<WsIoServerNamespace>> = LazyLock::new(|| {
    WS_IO_SERVER
        .new_namespace_builder("/sonic-rs")
        .unwrap()
        .on_connect(|_| async { Ok(()) })
        .packet_codec(WsIoPacketCodec::SonicRs)
        .register()
        .unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    POSTCARD.path();
    SONIC_RS.path();

    println!("Namespace count: {}", WS_IO_SERVER.namespace_count());
    let ws_io_layer = WS_IO_SERVER.layer();

    let app = Router::new().layer(ws_io_layer);
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
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
