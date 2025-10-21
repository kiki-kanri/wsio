use std::sync::LazyLock;
use std::process::id;
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
use wsio_server::WsIoServer;

// Constants/Statics
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
static MAIN_TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(TaskManager::new);
static WS_IO_SERVER: LazyLock<WsIoServer> = LazyLock::new(|| WsIoServer::builder().build());

#[tokio::main]
async fn main() -> Result<()> {
    println!("pid: {}", id());

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
