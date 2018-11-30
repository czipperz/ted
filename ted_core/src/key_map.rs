use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::Mutex;
use input::Input;
use state::State;
use display::Display;

/// Map [`Input`]s to [`KeyBind`]s.
///
/// [`Input`]: enum.Input.html
/// [`KeyBind`]: enum.KeyBind.html
pub struct KeyMap {
    bindings: HashMap<Input, KeyBind>,
    mappings: HashMap<Input, Vec<Input>>,
}

/// The action to be performed when the associated key is pressed.
#[derive(Clone)]
pub enum KeyBind {
    /// Run a function when the key is pressed.
    Action(Action),
    /// Have the user traverse a sub [`KeyMap`] when the key is pressed.
    ///
    /// [`KeyMap`]: struct.KeyMap.html
    SubMap(Arc<Mutex<KeyMap>>),
}

pub type Action = Arc<fn (&mut State, &mut Display) -> Result<(), ()>>;

impl KeyMap {
    /// This function performs the lookup.
    ///
    /// `Ok(action)` means an action was found and the inputs leading to
    /// it have been popped from the front of the `VecDeque`.
    ///
    /// `Err(true)` means an action was not found because not enough
    /// user input has been provided.  IE the user hits C-x and we
    /// stall for the next key (C-c for stop for instance).  We don't
    /// want to throw away those keys so `lookup_` will re-push the
    /// popped keys onto the beginning of the `VecDeque`.
    ///
    /// `Err(false)` means an action was not found because that key
    /// combination doesn't exist.
    pub fn lookup(key_map: &Arc<Mutex<KeyMap>>, inputs: &mut VecDeque<Input>,
                  throw_away: bool) -> Result<Action, bool> {
        match inputs.pop_front() {
            Some(input) => {
                let key_map = key_map.lock();
                match key_map.bindings.get(&input) {
                    Some(KeyBind::Action(action)) => Ok(action.clone()),
                    Some(KeyBind::SubMap(sub_map)) => {
                        match KeyMap::lookup(sub_map, inputs, throw_away) {
                            Ok(action) => Ok(action),
                            Err(e) => {
                                if !throw_away || e { inputs.push_front(input); }
                                Err(e)
                            },
                        }
                    },
                    None => Err(false),
                }
            },
            None => Err(true),
        }
    }

    /// Get what [`Input`]s a certain [`Input`] maps to.
    pub fn mapping(&self, input: &Input) -> Vec<Input> {
        match self.mappings.get(input) {
            Some(mapping) => mapping.iter().map(|m| self.mapping(m)).flatten().collect(),
            None => vec![input.clone()],
        }
    }

    /// map
    pub fn map(&mut self, input: Input, mapping: Vec<Input>) {
        self.mappings.insert(input, mapping);
    }

    /// Associate a [`KeyBind`] with an [`Input`]
    ///
    /// [`Input`]: enum.Input.html
    /// [`KeyBind`]: enum.KeyBind.html
    pub fn bind(&mut self, input: Input, binding: KeyBind) {
        self.bindings.insert(input, binding);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            bindings: HashMap::new(),
            mappings: HashMap::new(),
        }
    }
}
