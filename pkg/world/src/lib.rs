use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::{prelude::*, JsValue};

#[derive(Clone, Deserialize, Serialize)]
pub struct World {
    count: u32,
}

impl World {
    pub fn new() -> World {
        Self { count: 3 }
    }

    pub fn apply(self: &mut World, event: Event) {
        match event {
            Event::Increment => self.count += 1,
            Event::Decrement => self.count -= 1,
            Event::World(w) => *self = w,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Event {
    Increment,
    Decrement,
    World(World),
}

#[wasm_bindgen]
pub fn apply_event(world: JsValue, event: JsValue) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let event: Event = from_value(event)?;
    let mut world: World = from_value(world)?;
    world.apply(event);
    to_value(&world)
}
