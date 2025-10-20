use std::sync::{
    Arc,
    LazyLock,
};

use anyhow::Result;
use kikiutils::signal::wait_for_shutdown_signal;
use tokio::join;
use wsio_client::{
    WsIoClient,
    connection::WsIoClientConnection,
    core::packet::codecs::WsIoPacketCodec,
};

// Constants/Statics
static AUTH: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/auth")
        .unwrap()
        .auth::<&(), _, _>(|_| async { Ok(None) })
        .on_connection_close(|connection| on_connection_close(connection, "/auth"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/auth"))
        .build()
});

static BINCODE: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/bincode")
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, "/bincode"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/bincode"))
        .packet_codec(WsIoPacketCodec::Bincode)
        .build()
});

static CBOR: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/cbor")
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, "/cbor"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/cbor"))
        .packet_codec(WsIoPacketCodec::Cbor)
        .build()
});

static MSG_PACK: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/msg-pack")
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, "/msg-pack"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/msg-pack"))
        .packet_codec(WsIoPacketCodec::MsgPack)
        .build()
});

static SERDE_JSON: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/serde-json")
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, "/serde-json"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/serde-json"))
        .packet_codec(WsIoPacketCodec::SerdeJson)
        .build()
});

static SONIC_RS: LazyLock<WsIoClient> = LazyLock::new(|| {
    WsIoClient::builder("ws://127.0.0.1:8000/sonic-rs")
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, "/sonic-rs"))
        .on_connection_ready(|connection| on_connection_ready(connection, "/sonic-rs"))
        .packet_codec(WsIoPacketCodec::SonicRs)
        .build()
});

// Functions
async fn on_connection_close(_: Arc<WsIoClientConnection>, namespace: &str) -> Result<()> {
    println!("{}: on_connection_close", namespace);
    Ok(())
}

async fn on_connection_ready(_: Arc<WsIoClientConnection>, namespace: &str) -> Result<()> {
    println!("{}: on_connection_ready", namespace);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = join!(
        AUTH.connect(),
        BINCODE.connect(),
        CBOR.connect(),
        MSG_PACK.connect(),
        SERDE_JSON.connect(),
        SONIC_RS.connect(),
    );

    let _ = wait_for_shutdown_signal().await;
    let _ = join!(
        AUTH.disconnect(),
        BINCODE.disconnect(),
        CBOR.disconnect(),
        MSG_PACK.disconnect(),
        SERDE_JSON.disconnect(),
        SONIC_RS.disconnect(),
    );

    println!("Stopped");
    Ok(())
}
