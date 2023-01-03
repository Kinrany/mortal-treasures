mod player;

use std::{collections::HashSet, fmt::Debug, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::StreamExt;
use indexmap::IndexMap;
use mortal_treasures_world::{Event, World};
use player::Player;
use tokio::{spawn, sync::Mutex};
use tracing::{debug, instrument, warn};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Game(Arc<Mutex<Inner>>);

#[derive(Debug)]
struct Inner {
    worlds: IndexMap<Uuid, World>,
    players: IndexMap<Uuid, HashSet<Player>>,
}

const MAX_PLAYERS_PER_WORLD: usize = 3;

impl Inner {
    fn new() -> Self {
        Self {
            worlds: [].into(),
            players: [].into(),
        }
    }

    /// Create a new world and return it's ID.
    fn add_world(&mut self) -> Uuid {
        let uuid = Uuid::new_v4();
        self.worlds.insert(uuid, World::new());
        self.players.insert(uuid, [].into());
        uuid
    }

    fn player_world_mut(&mut self, player: &Player) -> (Uuid, &mut World) {
        let world = self
            .players
            .iter()
            .find_map(|(world, players)| players.contains(player).then_some(*world))
            .unwrap();
        (world, self.worlds.get_mut(&world).unwrap())
    }

    /// Add a player and return their assigned ID.
    fn add_player(&mut self, player: Player) -> Uuid {
        // Find the world to put the new player in.
        // Choose the first world, or create a new one.
        let world_id = self
            .players
            .iter()
            .find_map(|(world_id, players)| {
                (players.len() < MAX_PLAYERS_PER_WORLD).then_some(*world_id)
            })
            .unwrap_or_else(|| self.add_world());

        // Add the player.
        self.players.get_mut(&world_id).unwrap().insert(player);

        world_id
    }

    fn remove_player(&mut self, player: Player) {
        for players in self.players.values_mut() {
            players.remove(&player);
        }
    }

    fn world_players(&self, world: Uuid) -> &HashSet<Player> {
        self.players.get(&world).unwrap()
    }
}

impl Game {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::new())))
    }

    #[instrument(skip(f), level = "trace")]
    async fn with_lock<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut Inner) -> O,
    {
        f(&mut *self.0.lock().await)
    }

    async fn broadcast_to_world(&self, world: Uuid, event: &Event) {
        debug!(%world, ?event, "broadcasting");
        let players = self.with_lock(|g| g.world_players(world).clone()).await;
        for player in players {
            player.send_event(event).await;
        }
    }

    async fn handle_message_from_player(&self, player: &Player, event: Event) {
        let world_id = self
            .with_lock(|g| {
                let (world_id, world) = g.player_world_mut(player);
                world.apply(event.clone());
                world_id
            })
            .await;
        self.broadcast_to_world(world_id, &event).await;
    }

    pub async fn add_player(self, socket: WebSocket) -> Result<(), tokio::task::JoinError> {
        let (sender, mut receiver) = socket.split();

        let player = Player::new(sender);
        self.with_lock(|g| {
            let world_id = g.add_player(player.clone());
            let world = g.worlds.get(&world_id).unwrap().clone();
            let player = player.clone();
            spawn(async move { player.send_event(&Event::World(world)).await });
        })
        .await;

        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(msg) => {
                        debug!(?player, ?msg, "incoming message");
                        match msg {
                            Message::Text(text) => {
                                if let Ok(event) = serde_json::from_str::<Event>(&text) {
                                    self.handle_message_from_player(&player, event).await;
                                } else {
                                    warn!(%text, "not an event");
                                }
                            }
                            Message::Binary(bytes) => {
                                if let Ok(event) = serde_json::from_slice::<Event>(&bytes) {
                                    self.handle_message_from_player(&player, event).await;
                                } else {
                                    warn!(?bytes, "not an event");
                                }
                            }
                            Message::Ping(x) => player.send(Message::Pong(x)).await,
                            Message::Pong(_) => warn!("unexpected pong"),
                            Message::Close(_) => {
                                self.with_lock(|g| g.remove_player(player)).await;
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        warn!(?player, %error, "incoming websocket error");
                    }
                }
            }
        })
        .await
    }
}
