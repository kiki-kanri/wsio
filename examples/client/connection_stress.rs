use std::{
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use tokio::{
    sync::Semaphore,
    time::sleep,
};
use wsio_client::{
    WsIoClient,
    core::packet::codecs::WsIoPacketCodec,
};

// Constants/Statics
const CONNECT_CONCURRENCY: usize = 1000;
const CONNECTION_COUNT: usize = 10000;

#[tokio::main]
async fn main() -> Result<()> {
    let sem = Arc::new(Semaphore::new(CONNECT_CONCURRENCY));
    let mut handles = Vec::with_capacity(CONNECTION_COUNT);
    for i in 0..CONNECTION_COUNT {
        let permit = sem.clone().acquire_owned().await?;
        handles.push(tokio::spawn(async move {
            let client = WsIoClient::builder("ws://127.0.0.1:8000/bincode")
                .unwrap()
                .packet_codec(WsIoPacketCodec::Bincode)
                .build();

            let _ = client.connect().await;
            drop(permit);

            if i % 1000 == 0 {
                println!("connected {i}");
            }

            sleep(Duration::from_secs(10)).await;
            let _ = client.disconnect().await;

            if i % 1000 == 0 {
                println!("disconnected {i}");
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
