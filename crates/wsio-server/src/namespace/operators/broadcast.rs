use std::{
    collections::HashSet,
    sync::Arc,
};

use anyhow::Result;
use futures_util::{
    StreamExt,
    future::ready,
    stream::iter,
};
use serde::Serialize;

use super::super::{
    NamespaceStatus,
    WsIoServerNamespace,
};
use crate::{
    connection::WsIoServerConnection,
    core::{
        packet::WsIoPacket,
        types::hashers::FxHashSet,
    },
};

// Structs
pub struct WsIoServerNamespaceBroadcastOperator {
    exclude_rooms: HashSet<String>,
    include_rooms: HashSet<String>,
    namespace: Arc<WsIoServerNamespace>,
}

impl WsIoServerNamespaceBroadcastOperator {
    #[inline]
    pub(in super::super) fn new(namespace: Arc<WsIoServerNamespace>) -> Self {
        Self {
            exclude_rooms: HashSet::new(),
            include_rooms: HashSet::new(),
            namespace,
        }
    }

    // Private methods
    async fn for_each_target_connections<F, Fut>(&self, f: F)
    where
        F: Fn(Arc<WsIoServerConnection>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let mut target_connection_ids = FxHashSet::default();
        if self.include_rooms.is_empty() {
            target_connection_ids.extend(self.namespace.connections.iter().map(|entry| *entry.key()));
        } else {
            for room_name in &self.include_rooms {
                if let Some(room) = self.namespace.rooms.get(room_name) {
                    target_connection_ids.extend(room.iter().map(|entry| *entry.key()));
                }
            }
        };

        for room_name in &self.exclude_rooms {
            if let Some(room) = self.namespace.rooms.get(room_name) {
                for entry in room.iter() {
                    target_connection_ids.remove(&entry);
                }
            }
        }

        iter(target_connection_ids)
            .filter_map(|target_connection_id| {
                ready(
                    self.namespace
                        .connections
                        .get(&target_connection_id)
                        .map(|entry| entry.value().clone()),
                )
            })
            .for_each_concurrent(self.namespace.config.broadcast_concurrency_limit, |connection| async {
                let _ = f(connection).await;
            })
            .await;
    }

    // Public methods
    pub async fn disconnect(&self) -> Result<()> {
        let message = self.namespace.encode_packet_to_message(&WsIoPacket::new_disconnect())?;
        self.for_each_target_connections(move |connection| {
            let message = message.clone();
            async move { connection.send_message(message).await }
        })
        .await;

        Ok(())
    }

    pub async fn emit<D: Serialize>(&self, event: impl AsRef<str>, data: Option<&D>) -> Result<()> {
        self.namespace.status.ensure(NamespaceStatus::Running, |status| {
            format!("Cannot emit in invalid status: {status:?}")
        })?;

        let message = self.namespace.encode_packet_to_message(&WsIoPacket::new_event(
            event.as_ref(),
            data.map(|data| self.namespace.config.packet_codec.encode_data(data))
                .transpose()?,
        ))?;

        self.for_each_target_connections(move |connection| {
            let message = message.clone();
            async move { connection.emit_event_message(message).await }
        })
        .await;

        Ok(())
    }

    #[inline]
    pub fn except<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, room_names: I) -> Self {
        self.exclude_rooms
            .extend(room_names.into_iter().map(|room_name| room_name.as_ref().to_string()));

        self
    }

    #[inline]
    pub fn to<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, room_names: I) -> Self {
        self.include_rooms
            .extend(room_names.into_iter().map(|room_name| room_name.as_ref().to_string()));

        self
    }
}
