use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Game(Arc<Mutex<Vec<World>>>);

impl Game {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(vec![World::new()])))
    }

    pub async fn with_first_world<T>(&self, f: impl FnOnce(&mut World) -> T) -> T {
        f(&mut self.0.lock().await[0])
    }

    pub async fn apply_to_first(&self, event: Event) {
        self.0.lock().await[0].apply(event);
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
