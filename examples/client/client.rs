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
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .auth(|_| async { Ok(()) })
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static BINCODE: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/bincode";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::Bincode)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static CBOR: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/cbor";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::Cbor)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static DISCONNECT: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/disconnect";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static MSG_PACK: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/msgpack";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::MsgPack)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static POSTCARD: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/postcard";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::Postcard)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static SERDE_JSON: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/serde-json";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::SerdeJson)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
});

static SONIC_RS: LazyLock<WsIoClient> = LazyLock::new(|| {
    const NAMESPACE: &str = "/sonic-rs";
    let client = WsIoClient::builder(format!("ws://127.0.0.1:8000/{NAMESPACE}").as_str())
        .unwrap()
        .on_connection_close(|connection| on_connection_close(connection, NAMESPACE))
        .on_connection_ready(|connection| on_connection_ready(connection, NAMESPACE))
        .packet_codec(WsIoPacketCodec::SonicRs)
        .build();

    client.on("test", |_, _: Arc<()>| on_event(NAMESPACE));
    client
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

async fn on_event(namespace: &str) -> Result<()> {
    println!("{namespace}: on_event");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    join!(
        AUTH.connect(),
        BINCODE.connect(),
        CBOR.connect(),
        DISCONNECT.connect(),
        MSG_PACK.connect(),
        POSTCARD.connect(),
        SERDE_JSON.connect(),
        SONIC_RS.connect(),
    );

    let _ = wait_for_shutdown_signal().await;
    join!(
        AUTH.disconnect(),
        BINCODE.disconnect(),
        CBOR.disconnect(),
        DISCONNECT.disconnect(),
        MSG_PACK.disconnect(),
        POSTCARD.disconnect(),
        SERDE_JSON.disconnect(),
        SONIC_RS.disconnect(),
    );

    println!("Stopped");
    Ok(())
}
