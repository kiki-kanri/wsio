use std::{
    any::{
        Any,
        TypeId,
    },
    collections::{
        HashMap,
        hash_map::Entry,
    },
    pin::Pin,
    sync::{
        Arc,
        LazyLock,
        atomic::{
            AtomicU32,
            Ordering,
        },
    },
};

use anyhow::Result;
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use tokio::spawn;

use crate::packet::codecs::WsIoPacketCodec;

// Types
type DataDecoder = fn(&[u8], &WsIoPacketCodec) -> Result<Arc<dyn Any + Send + Sync>>;
type Handler<T> = Arc<
    dyn Fn(Arc<T>, Arc<dyn Any + Send + Sync>) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

// Structs
struct EventEntry<T> {
    data_decoder: DataDecoder,
    data_type_id: TypeId,
    handlers: RwLock<HashMap<u32, Handler<T>>>,
}

pub struct WsIoEventRegistry<T: Send + Sync + 'static> {
    event_entries: RwLock<HashMap<String, Arc<EventEntry<T>>>>,
    next_handler_id: AtomicU32,
    packet_codec: WsIoPacketCodec,
}

impl<T: Send + Sync + 'static> WsIoEventRegistry<T> {
    #[inline]
    pub fn new(packet_codec: WsIoPacketCodec) -> Self {
        Self {
            event_entries: RwLock::new(HashMap::new()),
            next_handler_id: AtomicU32::new(0),
            packet_codec,
        }
    }

    // Public methods
    #[inline]
    pub fn dispatch_event_packet(&self, t: Arc<T>, event: impl AsRef<str>, packet_data: Option<Vec<u8>>) {
        let Some(event_entry) = self.event_entries.read().get(event.as_ref()).cloned() else {
            return;
        };

        let packet_codec = self.packet_codec;
        // TODO: task manager
        spawn(async move {
            let data = match packet_data {
                Some(bytes) => match (event_entry.data_decoder)(&bytes, &packet_codec) {
                    Ok(data) => data,
                    Err(_) => return,
                },
                None => EMPTY_EVENT_DATA_ANY_ARC.clone(),
            };

            for handler in event_entry.handlers.read().values() {
                let data = data.clone();
                let handler = handler.clone();
                let t = t.clone();
                spawn(async move {
                    let _ = handler(t, data).await;
                });
            }
        });
    }

    #[inline]
    pub fn off(&self, event: impl AsRef<str>) {
        self.event_entries.write().remove(event.as_ref());
    }

    #[inline]
    pub fn off_by_handler_id(&self, event: impl AsRef<str>, handler_id: u32) {
        let event = event.as_ref();
        let mut event_entries = self.event_entries.write();
        let remove_event = if let Some(event_entry) = event_entries.get_mut(event) {
            let mut handlers = event_entry.handlers.write();
            handlers.remove(&handler_id);
            handlers.is_empty()
        } else {
            false
        };

        if remove_event {
            event_entries.remove(event);
        }
    }

    #[inline]
    pub fn on<H, Fut, D>(&self, event: impl Into<String>, handler: H) -> u32
    where
        H: Fn(Arc<T>, Arc<D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + Sync + 'static,
    {
        let data_type_id = TypeId::of::<D>();
        let event = event.into();

        let mut event_entries = self.event_entries.write();
        let event_entry = match event_entries.entry(event.clone()) {
            Entry::Occupied(occupied) => {
                let event_entry = occupied.into_mut();
                assert_eq!(
                    event_entry.data_type_id, data_type_id,
                    "Event '{}' already registered with a different data type — each event name must correspond to exactly one payload type.",
                    event
                );

                event_entry
            }
            Entry::Vacant(vacant) => vacant.insert(Arc::new(EventEntry {
                data_decoder: decode_data_as_any_arc::<D>,
                data_type_id,
                handlers: RwLock::new(HashMap::new()),
            })),
        };

        let handler_id = self.next_handler_id.fetch_add(1, Ordering::Relaxed);
        event_entry.handlers.write().insert(
            handler_id,
            Arc::new(move |connection, data| {
                if (*data).type_id() != data_type_id {
                    return Box::pin(async { Ok(()) });
                }

                Box::pin(handler(connection, data.downcast().unwrap()))
            }),
        );

        handler_id
    }
}

// Constants/Statics
static EMPTY_EVENT_DATA_ANY_ARC: LazyLock<Arc<dyn Any + Send + Sync>> = LazyLock::new(|| Arc::new(()));

// Functions
#[inline]
fn decode_data_as_any_arc<D: DeserializeOwned + Send + Sync + 'static>(
    bytes: &[u8],
    packet_codec: &WsIoPacketCodec,
) -> Result<Arc<dyn Any + Send + Sync>> {
    Ok(Arc::new(packet_codec.decode_data::<D>(bytes)?))
}
