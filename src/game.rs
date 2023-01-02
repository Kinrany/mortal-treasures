use std::{fmt::Display, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, Sink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct Game(Arc<Mutex<Inner>>);

struct Inner {
    worlds: Vec<World>,
    players: Vec<Player>,
}

type Player = SplitSink<WebSocket, Message>;

impl Inner {
    fn new() -> Self {
        Self {
            worlds: vec![World::new()],
            players: vec![],
        }
    }

    fn first_world_mut(&mut self) -> &mut World {
        &mut self.worlds[0]
    }

    async fn add_player(&mut self, player: Player) {
        let id = self.players.len();
        self.players.push(player);
        let world = self.first_world_mut().clone();
        send_event(&mut self.players[id], &Event::World(world)).await;
    }

    async fn broadcast(&mut self, event: &Event) {
        for player in &mut self.players {
            send_event(player, event).await
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::new())))
    }

    pub async fn add_player(self, socket: WebSocket) -> Result<(), tokio::task::JoinError> {
        let (sender, mut receiver) = socket.split();

        self.0.lock().await.add_player(sender).await;

        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        let message = msg.into_text().unwrap();
                        debug!(%message, "incoming message");
                        let Ok(event) = serde_json::from_str::<Event>(&message) else {
                            warn!("not an event");
                            continue;
                        };
                        let mut g = self.0.lock().await;
                        g.first_world_mut().apply(event.clone());
                        g.broadcast(&event).await;
                    }
                    Err(error) => {
                        warn!(%error, "incoming websocket error");
                    }
                }
            }
        })
        .await
    }
}

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
