use std::{fmt::Display, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{Sink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct Game(Arc<Mutex<(Vec<World>,)>>);

impl Game {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new((vec![World::new()],))))
    }

    pub async fn with_first_world<T>(&self, f: impl FnOnce(&mut World) -> T) -> T {
        f(&mut self.0.lock().await.0[0])
    }

    pub async fn add_player(self, socket: WebSocket) -> Result<(), tokio::task::JoinError> {
        tokio::spawn(async move {
            let (mut sender, mut receiver) = socket.split();

            async fn send<S, M>(sink: &mut S, m: M)
            where
                S: Sink<M> + Unpin,
                S::Error: Display,
            {
                if let Err(error) = sink.send(m).await {
                    error!(%error, "outgoing websocket error");
                }
            }

            async fn send_event<S>(sink: &mut S, e: &Event)
            where
                S: Sink<Message> + Unpin,
                S::Error: Display,
            {
                let m: Message = serde_json::to_string(e).unwrap().into();
                send(sink, m).await
            }

            send_event(
                &mut sender,
                &Event::World(self.with_first_world(|w| w.clone()).await),
            )
            .await;

            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        let message = msg.into_text().unwrap();
                        debug!(%message, "incoming message");
                        let Ok(event) = serde_json::from_str::<Event>(&message) else {
                            warn!("not an event");
                            continue;
                        };
                        self.with_first_world(|e| e.apply(event.clone())).await;
                        send_event(&mut sender, &event).await;
                    }
                    Err(error) => {
                        warn!(%error, "incoming websocket error");
                        send(&mut sender, error.to_string().into()).await;
                    }
                }
            }
        })
        .await
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct World {
    count: u32,
}

impl World {
    pub fn new() -> Self {
        Self { count: 3 }
    }

    pub fn apply(&mut self, event: Event) {
        match event {
            Event::Increment => self.count += 1,
            Event::Decrement => self.count -= 1,
            Event::World(w) => *self = w,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Event {
    Increment,
    Decrement,
    World(World),
}
