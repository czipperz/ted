use key_map::KeyMap;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct Mode {
    pub key_map: Arc<Mutex<KeyMap>>,
    pub fallthrough_behavior: FallthroughBehavior,
}

impl Mode {
    pub fn new() -> Self {
        Mode::default()
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::from(Arc::default())
    }
}

impl From<Arc<Mutex<KeyMap>>> for Mode {
    fn from(key_map: Arc<Mutex<KeyMap>>) -> Self {
        Mode {
            key_map,
            fallthrough_behavior: FallthroughBehavior::Fallthrough,
        }
    }
}

pub enum FallthroughBehavior {
    Fallthrough,
    Error,
}
