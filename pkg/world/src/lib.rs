use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::{prelude::*, JsValue};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct World {
    pub count: i32,
    pub text: String,
}

impl World {
    pub fn new() -> World {
        Self {
            count: 3,
            text: "hello".to_string(),
        }
    }

    pub fn apply(self: &mut World, event: Event) {
        match event {
            Event::World(w) => *self = w,
            Event::Increment => self.count += 1,
            Event::Decrement => self.count -= 1,
            Event::Text { s } => self.text = s,
            Event::GameOver => (),
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Event {
    World(World),
    Increment,
    Decrement,
    Text { s: String },
    GameOver,
}

#[wasm_bindgen]
pub fn apply_event(world: JsValue, event: JsValue) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let event: Event = from_value(event)?;
    let mut world: World = from_value(world)?;
    world.apply(event);
    to_value(&world)
}
