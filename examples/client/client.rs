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
    const NAMESPACE: &str = "/auth";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .auth::<&(), _, _>(|_| async { Ok(None) })
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .build()
});

static BINCODE: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/bincode";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::Bincode)
        .build()
});

static CBOR: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/cbor";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::Cbor)
        .build()
});

static DISCONNECT: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/disconnect";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .build()
});

static MSG_PACK: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/msgpack";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::MsgPack)
        .build()
});

static SERDE_JSON: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/serde-json";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::SerdeJson)
        .build()
});

static SONIC_RS: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/sonic-rs";
    WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::SonicRs)
        .build()
});

// Functions
async fn on_connection_close(_: Arc<WsIoClientConnection>, namespace: &str) -> Result<()> {
    println!("{namespace}: on_connection_close");
    Ok(())
}

async fn on_connection_ready(_: Arc<WsIoClientConnection>, namespace: &str) -> Result<()> {
    println!("{namespace}: on_connection_ready");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = join!(
        AUTH.connect(),
        BINCODE.connect(),
        CBOR.connect(),
        DISCONNECT.connect(),
        MSG_PACK.connect(),
        SERDE_JSON.connect(),
        SONIC_RS.connect(),
    );

    let _ = wait_for_shutdown_signal().await;
    let _ = join!(
        AUTH.disconnect(),
        BINCODE.disconnect(),
        CBOR.disconnect(),
        DISCONNECT.disconnect(),
        MSG_PACK.disconnect(),
        SERDE_JSON.disconnect(),
        SONIC_RS.disconnect(),
    );

    println!("Stopped");
    Ok(())
}
