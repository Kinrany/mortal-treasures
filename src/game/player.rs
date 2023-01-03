use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use mortal_treasures_world::Event;

use tokio::sync::Mutex;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct Player {
    uuid: Uuid,
    sink: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

impl Player {
    pub fn new(sink: SplitSink<WebSocket, Message>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            sink: Arc::new(Mutex::new(sink)),
        }
    }

    pub async fn send(&self, message: Message) {
        if let Err(error) = self.sink.lock().await.send(message).await {
            error!(%error, "outgoing websocket error");
        }
    }

    pub async fn send_event(&self, event: &Event) {
        let message = serde_json::to_string(event).unwrap().into();
        self.send(message).await;
    }
}

impl Hash for Player {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}
impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}
impl Eq for Player {}

impl Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Player").field(&self.uuid).finish()
    }
}
