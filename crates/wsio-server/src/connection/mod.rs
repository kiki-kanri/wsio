use std::sync::{
    Arc,
    LazyLock,
    atomic::{
        AtomicU64,
        Ordering,
    },
};

use anyhow::{
    Result,
    bail,
};
use arc_swap::ArcSwap;
use http::{
    HeaderMap,
    Uri,
};
use num_enum::{
    IntoPrimitive,
    TryFromPrimitive,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use tokio::{
    spawn,
    sync::{
        Mutex,
        mpsc::{
            Receiver,
            Sender,
            channel,
        },
    },
    task::JoinHandle,
    time::{
        sleep,
        timeout,
    },
};
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

#[cfg(feature = "connection-extensions")]
mod extensions;

#[cfg(feature = "connection-extensions")]
use self::extensions::ConnectionExtensions;
use crate::{
    WsIoServer,
    core::{
        atomic::status::AtomicStatus,
        channel_capacity_from_websocket_config,
        event::registry::WsIoEventRegistry,
        packet::{
            WsIoPacket,
            WsIoPacketType,
        },
        traits::task::spawner::TaskSpawner,
        types::{
            BoxAsyncUnaryResultHandler,
            hashers::FxDashSet,
        },
        utils::task::abort_locked_task,
    },
    namespace::WsIoServerNamespace,
};

// Enums
#[repr(u8)]
#[derive(Debug, Eq, IntoPrimitive, PartialEq, TryFromPrimitive)]
enum ConnectionStatus {
    Activating,
    AwaitingInit,
    Closed,
    Closing,
    Created,
    Initiating,
    Ready,
}

// Structs
pub struct WsIoServerConnection {
    cancel_token: ArcSwap<CancellationToken>,
    event_registry: WsIoEventRegistry<WsIoServerConnection, WsIoServerConnection>,
    #[cfg(feature = "connection-extensions")]
    extensions: ConnectionExtensions,
    headers: HeaderMap,
    id: u64,
    init_timeout_task: Mutex<Option<JoinHandle<()>>>,
    joined_rooms: FxDashSet<String>,
    message_tx: Sender<Arc<Message>>,
    namespace: Arc<WsIoServerNamespace>,
    on_close_handler: Mutex<Option<BoxAsyncUnaryResultHandler<Self>>>,
    request_uri: Uri,
    status: AtomicStatus<ConnectionStatus>,
}

impl TaskSpawner for WsIoServerConnection {
    #[inline]
    fn cancel_token(&self) -> Arc<CancellationToken> {
        self.cancel_token.load_full()
    }
}

impl WsIoServerConnection {
    #[inline]
    pub(crate) fn new(
        headers: HeaderMap,
        namespace: Arc<WsIoServerNamespace>,
        request_uri: Uri,
    ) -> (Arc<Self>, Receiver<Arc<Message>>) {
        let channel_capacity = channel_capacity_from_websocket_config(&namespace.config.websocket_config);
        let (message_tx, message_rx) = channel(channel_capacity);
        (
            Arc::new(Self {
                cancel_token: ArcSwap::new(Arc::new(CancellationToken::new())),
                event_registry: WsIoEventRegistry::new(),
                #[cfg(feature = "connection-extensions")]
                extensions: ConnectionExtensions::new(),
                headers,
                id: NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed),
                init_timeout_task: Mutex::new(None),
                joined_rooms: FxDashSet::default(),
                message_tx,
                namespace,
                on_close_handler: Mutex::new(None),
                request_uri,
                status: AtomicStatus::new(ConnectionStatus::Created),
            }),
            message_rx,
        )
    }

    // Private methods
    #[inline]
    fn handle_event_packet(self: &Arc<Self>, event: &str, packet_data: Option<Vec<u8>>) -> Result<()> {
        self.event_registry.dispatch_event_packet(
            self.clone(),
            event,
            &self.namespace.config.packet_codec,
            packet_data,
            self,
        );

        Ok(())
    }

    async fn handle_init_packet(self: &Arc<Self>, packet_data: Option<&[u8]>) -> Result<()> {
        // Verify current state; only valid from AwaitingInit → Initiating
        let status = self.status.get();
        match status {
            ConnectionStatus::AwaitingInit => self.status.try_transition(status, ConnectionStatus::Initiating)?,
            _ => bail!("Received init packet in invalid status: {:#?}", status),
        }

        // Abort init-timeout task if still active
        abort_locked_task(&self.init_timeout_task).await;

        // Invoke init_response handler with timeout protection if configured
        if let Some(init_response_handler) = &self.namespace.config.init_response_handler {
            timeout(
                self.namespace.config.init_response_handler_timeout,
                init_response_handler(self.clone(), packet_data, &self.namespace.config.packet_codec),
            )
            .await??
        }

        // Activate connection
        self.status
            .try_transition(ConnectionStatus::Initiating, ConnectionStatus::Activating)?;

        // Invoke middleware with timeout protection if configured
        if let Some(middleware) = &self.namespace.config.middleware {
            timeout(
                self.namespace.config.middleware_execution_timeout,
                middleware(self.clone()),
            )
            .await??;

            // Ensure connection is still in Activating state
            self.status.ensure(ConnectionStatus::Activating, |status| {
                format!("Cannot activate connection in invalid status: {:#?}", status)
            })?;
        }

        // Invoke on_connect handler with timeout protection if configured
        if let Some(on_connect_handler) = &self.namespace.config.on_connect_handler {
            timeout(
                self.namespace.config.on_connect_handler_timeout,
                on_connect_handler(self.clone()),
            )
            .await??;
        }

        // Transition state to Ready
        self.status
            .try_transition(ConnectionStatus::Activating, ConnectionStatus::Ready)?;

        // Insert connection into namespace
        self.namespace.insert_connection(self.clone());

        // Send ready packet
        self.send_packet(&WsIoPacket::new_ready()).await?;

        // Invoke on_ready handler if configured
        if let Some(on_ready_handler) = self.namespace.config.on_ready_handler.clone() {
            // Run handler asynchronously in a detached task
            self.spawn_task(on_ready_handler(self.clone()));
        }

        Ok(())
    }

    async fn send_packet(&self, packet: &WsIoPacket) -> Result<()> {
        self.send_message(self.namespace.encode_packet_to_message(packet)?)
            .await
    }

    // Protected methods
    pub(crate) async fn cleanup(self: &Arc<Self>) {
        // Set connection state to Closing
        self.status.store(ConnectionStatus::Closing);

        // Remove connection from namespace
        self.namespace.remove_connection(self.id);

        // Leave all joined rooms
        let joined_rooms = self.joined_rooms.iter().map(|entry| entry.clone()).collect::<Vec<_>>();
        for room_name in &joined_rooms {
            self.namespace.remove_connection_id_from_room(room_name, self.id);
        }

        self.joined_rooms.clear();

        // Abort init-timeout task if still active
        abort_locked_task(&self.init_timeout_task).await;

        // Cancel all ongoing operations via cancel token
        self.cancel_token.load().cancel();

        // Invoke on_close handler with timeout protection if configured
        if let Some(on_close_handler) = self.on_close_handler.lock().await.take() {
            let _ = timeout(
                self.namespace.config.on_close_handler_timeout,
                on_close_handler(self.clone()),
            )
            .await;
        }

        // Set connection state to Closed
        self.status.store(ConnectionStatus::Closed);
    }

    #[inline]
    pub(crate) fn close(&self) {
        // Skip if connection is already Closing or Closed, otherwise set connection state to Closing
        match self.status.get() {
            ConnectionStatus::Closed | ConnectionStatus::Closing => return,
            _ => self.status.store(ConnectionStatus::Closing),
        }

        // Send websocket close frame to initiate graceful shutdown
        let _ = self.message_tx.try_send(Arc::new(Message::Close(None)));
    }

    pub(crate) async fn emit_event_message(&self, message: Arc<Message>) -> Result<()> {
        self.status.ensure(ConnectionStatus::Ready, |status| {
            format!("Cannot emit in invalid status: {:#?}", status)
        })?;

        self.send_message(message).await
    }

    pub(crate) async fn handle_incoming_packet(self: &Arc<Self>, bytes: &[u8]) -> Result<()> {
        // TODO: lazy load
        let packet = self.namespace.config.packet_codec.decode(bytes)?;
        match packet.r#type {
            WsIoPacketType::Event => {
                if let Some(event) = packet.key.as_deref() {
                    self.handle_event_packet(event, packet.data)
                } else {
                    bail!("Event packet missing key");
                }
            }
            WsIoPacketType::Init => self.handle_init_packet(packet.data.as_deref()).await,
            _ => Ok(()),
        }
    }

    pub(crate) async fn init(self: &Arc<Self>) -> Result<()> {
        // Verify current state; only valid Created
        self.status.ensure(ConnectionStatus::Created, |status| {
            format!("Cannot init connection in invalid status: {:#?}", status)
        })?;

        // Generate init request data if init request handler is configured
        let init_request_data = if let Some(init_request_handler) = &self.namespace.config.init_request_handler {
            timeout(
                self.namespace.config.init_request_handler_timeout,
                init_request_handler(self.clone(), &self.namespace.config.packet_codec),
            )
            .await??
        } else {
            None
        };

        // Transition state to AwaitingInit
        self.status
            .try_transition(ConnectionStatus::Created, ConnectionStatus::AwaitingInit)?;

        // Spawn init-response-timeout watchdog to close connection if init not received in time
        let connection = self.clone();
        *self.init_timeout_task.lock().await = Some(spawn(async move {
            sleep(connection.namespace.config.init_response_timeout).await;
            if connection.status.is(ConnectionStatus::AwaitingInit) {
                connection.close();
            }
        }));

        // Send init packet
        self.send_packet(&WsIoPacket::new_init(init_request_data)).await
    }

    pub(crate) async fn send_message(&self, message: Arc<Message>) -> Result<()> {
        Ok(self.message_tx.send(message).await?)
    }

    // Public methods
    pub async fn disconnect(&self) {
        let _ = self.send_packet(&WsIoPacket::new_disconnect()).await;
        self.close()
    }

    pub async fn emit<D: Serialize>(&self, event: impl AsRef<str>, data: Option<&D>) -> Result<()> {
        self.emit_event_message(
            self.namespace.encode_packet_to_message(&WsIoPacket::new_event(
                event.as_ref(),
                data.map(|data| self.namespace.config.packet_codec.encode_data(data))
                    .transpose()?,
            ))?,
        )
        .await
    }

    #[cfg(feature = "connection-extensions")]
    #[inline]
    pub fn extensions(&self) -> &ConnectionExtensions {
        &self.extensions
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    #[inline]
    pub fn join<I: IntoIterator<Item = S>, S: AsRef<str>>(self: &Arc<Self>, room_names: I) {
        for room_name in room_names {
            let room_name = room_name.as_ref();
            self.namespace.add_connection_id_to_room(room_name, self.id);
            self.joined_rooms.insert(room_name.to_string());
        }
    }

    #[inline]
    pub fn leave<I: IntoIterator<Item = S>, S: AsRef<str>>(self: &Arc<Self>, room_names: I) {
        for room_name in room_names {
            self.namespace
                .remove_connection_id_from_room(room_name.as_ref(), self.id);

            self.joined_rooms.remove(room_name.as_ref());
        }
    }

    #[inline]
    pub fn namespace(&self) -> Arc<WsIoServerNamespace> {
        self.namespace.clone()
    }

    #[inline]
    pub fn off(&self, event: impl AsRef<str>) {
        self.event_registry.off(event.as_ref());
    }

    #[inline]
    pub fn off_by_handler_id(&self, event: impl AsRef<str>, handler_id: u32) {
        self.event_registry.off_by_handler_id(event.as_ref(), handler_id);
    }

    #[inline]
    pub fn on<H, Fut, D>(&self, event: impl AsRef<str>, handler: H) -> u32
    where
        H: Fn(Arc<WsIoServerConnection>, Arc<D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        D: DeserializeOwned + Send + Sync + 'static,
    {
        self.event_registry.on(event.as_ref(), handler)
    }

    pub async fn on_close<H, Fut>(&self, handler: H)
    where
        H: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        *self.on_close_handler.lock().await = Some(Box::new(move |connection| Box::pin(handler(connection))));
    }

    #[inline]
    pub fn request_uri(&self) -> &Uri {
        &self.request_uri
    }

    #[inline]
    pub fn server(&self) -> WsIoServer {
        self.namespace.server()
    }
}

// Constants/Statics
static NEXT_CONNECTION_ID: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
