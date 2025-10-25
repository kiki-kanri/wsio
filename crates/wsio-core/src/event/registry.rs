use std::{
    any::{
        Any,
        TypeId,
    },
    collections::HashMap,
    pin::Pin,
    sync::{
        Arc,
        RwLock,
        atomic::{
            AtomicU32,
            Ordering,
        },
    },
};

use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::packet::codecs::WsIoPacketCodec;

type DataDecoder =
    Box<dyn Fn(&[u8], &WsIoPacketCodec) -> Result<Box<dyn Any + Send + Sync + 'static>> + Send + Sync + 'static>;

type Handler<T> = Box<
    dyn for<'a> Fn(Arc<T>, &'a (dyn Any + Send + Sync + 'a)) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

pub struct WsIoEventRegistry<T> {
    pub data_decoders: RwLock<HashMap<String, DataDecoder>>,
    pub data_type_ids: RwLock<HashMap<String, TypeId>>,
    pub handlers: RwLock<HashMap<String, HashMap<u32, Handler<T>>>>,
    pub next_handler_id: AtomicU32,
}

impl<T> Default for WsIoEventRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> WsIoEventRegistry<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            data_decoders: RwLock::new(HashMap::new()),
            data_type_ids: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            next_handler_id: AtomicU32::new(0),
        }
    }

    // Public methods
    #[inline]
    pub fn off(&self, event: impl Into<String>) {
        let event = event.into();
        self.data_decoders.write().unwrap().remove(&event);
        self.data_type_ids.write().unwrap().remove(&event);
        self.handlers.write().unwrap().remove(&event);
    }

    #[inline]
    pub fn off_by_handler_id(&self, event: impl Into<String>, handler_id: u32) {
        let event = event.into();
        let mut handlers_map = self.handlers.write().unwrap();
        if let Some(handlers) = handlers_map.get_mut(&event) {
            handlers.remove(&handler_id);
            if handlers.is_empty() {
                drop(handlers_map);
                self.off(event);
            }
        }
    }

    #[inline]
    pub fn on<H, Fut, D>(&self, event: impl Into<String>, handler: H) -> u32
    where
        H: Fn(Arc<T>, &D) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + Sync + 'static,
    {
        // Resolve the concrete TypeId of the handler payload type `D`
        let data_type_id = TypeId::of::<D>();
        let event = event.into();

        // --------------------------------------------------------------------
        // Validate data type consistency
        // Each event name can only be associated with a single payload type `D`.
        // If a handler for the same event name was previously registered with
        // a different `D`, this indicates a programming error — panic early.
        // --------------------------------------------------------------------
        if let Some(type_id) = self.data_type_ids.read().unwrap().get(&event)
            && *type_id != data_type_id
        {
            panic!(
                "Event '{}' already registered with different data type — each event name must correspond to exactly one payload type.",
                &event
            );
        } else {
            // ----------------------------------------------------------------
            // Register the payload type and its decoder
            // For a new event name, store both:
            //   1. The TypeId of its associated payload type `D`
            //   2. A dynamic decoder closure capable of deserializing bytes
            //      into `D` via the configured `WsIoPacketCodec`
            // ----------------------------------------------------------------
            self.data_type_ids.write().unwrap().insert(event.clone(), data_type_id);
            self.data_decoders.write().unwrap().insert(
                event.clone(),
                Box::new(|bytes, packet_codec| Ok(Box::new(packet_codec.decode_data::<D>(bytes)?))),
            );
        }

        // --------------------------------------------------------------------
        // Register the handler
        // Acquire a unique handler ID and insert the user-provided closure
        // into the event’s handler map. Each handler is wrapped in a boxed
        // dynamic async function that:
        //   - Receives an `Arc<T>` connection reference
        //   - Downcasts the dynamically-typed data back to `&D`
        //   - Executes the original user handler asynchronously
        // --------------------------------------------------------------------
        let mut handlers = self.handlers.write().unwrap();
        let handlers = handlers.entry(event).or_default();
        let handler_id = self.next_handler_id.fetch_add(1, Ordering::Relaxed);
        handlers.insert(
            handler_id,
            Box::new(move |connection, data| Box::pin(handler(connection, data.downcast_ref().unwrap()))),
        );

        handler_id
    }
}
