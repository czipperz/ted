use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use input::Input;
use state::State;
use display::Display;

pub struct KeyMap {
    map: HashMap<Input, KeyBind>,
}

#[derive(Clone)]
pub enum KeyBind {
    Action(Arc<fn (&mut State, &mut Display) -> Result<(), ()>>),
    SubMap(Arc<Mutex<KeyMap>>),
}

impl KeyMap {
    pub fn lookup(&self, input: &Input) -> Option<&KeyBind> {
        self.map.get(input)
    }

    pub fn bind(&mut self, input: Input, bind: KeyBind) {
        self.map.insert(input, bind);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap { map: HashMap::new() }
    }
}
