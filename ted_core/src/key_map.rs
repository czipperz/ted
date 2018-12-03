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
}

/// The action to be performed when the associated key is pressed.
#[derive(Clone)]
pub enum KeyBind {
    /// Run a function when the key is pressed.
    Action(Action),
    /// Traverse a sub [`KeyMap`] when the key is pressed.
    ///
    /// [`KeyMap`]: struct.KeyMap.html
    SubMap(Arc<Mutex<KeyMap>>),
    /// This key is bound to another sequence of keys.
    Mapping(Vec<Input>),
}

/// A command to be ran when a certain sequence of keys are pressed.
///
/// A command must allow account for the fact that multiple commands
/// can potentially run at once.  Thus, if the action is only to edit
/// one [`Buffer`], it should not keep the entire [`State`] locked.  It
/// should lock the [`State`] only to retrieve the [`Buffer`] to edit.
/// Then the [`State`] should be unlocked and just the [`Buffer`] should
/// be locked.  But it should only be locked when being updated.  Thus
/// work can be done by multiple workers at the same time.
///
/// [`Buffer`]: struct.Buffer.html
/// [`State`]: struct.State.html
pub type Action = Arc<fn (Arc<Mutex<State>>, Arc<Mutex<Display>>) -> Result<(), ()>>;

impl KeyMap {
    /// This function performs the lookup.
    ///
    /// `Ok(action)` means an action was found and the inputs leading to
    /// it have been popped from the front of the `VecDeque`.
    ///
    /// `Err(true)` means an action was not found because not enough
    /// user input has been provided.  IE the user hits `C-x` and we
    /// stall for the next key (`C-c` for stop for instance).  We don't
    /// want to throw away those keys so `lookup_` will re-push the
    /// popped keys onto the beginning of the `VecDeque`.
    ///
    /// `Err(false)` means an action was not found because that key
    /// combination doesn't exist.
    pub fn lookup(key_map: &Arc<Mutex<KeyMap>>, inputs: &mut VecDeque<Input>,
                  throw_away: bool) -> Result<Action, bool> {
        loop {
            match KeyMap::lookup_(key_map, inputs, throw_away)? {
                Ok(action) => return Ok(action),
                // there was a KeyBind::Mapping encountered and we must restart the search
                Err(()) => (),
            }
        }
    }

    /// Bind a sequence of [`Input`]s to run a certain [`Action`].
    ///
    /// [`Input`]: enum.Input.html
    /// [`Action`]: type.Action.html
    pub fn bind(&mut self, inputs: Vec<Input>, action: Action) {
        self.assign(&mut inputs.into(), KeyBind::Action(action))
    }

    /// Bind a sequence of [`Input`]s to a sub `KeyMap`.
    ///
    /// [`Input`]: enum.Input.html
    pub fn bind_key_map(&mut self, inputs: Vec<Input>, key_map: Arc<Mutex<KeyMap>>) {
        self.assign(&mut inputs.into(), KeyBind::SubMap(key_map))
    }

    /// Bind a sequence of [`Input`]s to another sequence of [`Input`]s.
    ///
    /// [`Input`]: enum.Input.html
    pub fn map(&mut self, inputs: Vec<Input>, mapping: Vec<Input>) {
        self.assign(&mut inputs.into(), KeyBind::Mapping(mapping))
    }

    fn lookup_(key_map: &Arc<Mutex<KeyMap>>, inputs: &mut VecDeque<Input>,
               throw_away: bool) -> Result<Result<Action, ()>, bool> {
        match inputs.pop_front() {
            Some(input) => {
                let key_map = key_map.lock();
                match key_map.bindings.get(&input) {
                    Some(KeyBind::Action(action)) => Ok(Ok(action.clone())),
                    Some(KeyBind::SubMap(sub_map)) => {
                        KeyMap::lookup_(sub_map, inputs, throw_away).map_err(|e| {
                            if !throw_away || e { inputs.push_front(input); }
                            e
                        })
                    },
                    Some(KeyBind::Mapping(mapping)) => {
                        for m in mapping.into_iter().rev() {
                            inputs.push_front(*m)
                        }
                        Ok(Err(()))
                    },
                    None => Err(false),
                }
            },
            None => Err(true),
        }
    }

    fn assign(&mut self, inputs: &mut VecDeque<Input>, mut key_bind: KeyBind) {
        loop {
            match self.assign_(inputs, key_bind) {
                Ok(()) => return,
                Err(kb) => key_bind = kb,
            }
        }
    }

    fn assign_(&mut self, inputs: &mut VecDeque<Input>, key_bind: KeyBind) -> Result<(), KeyBind> {
        fn create_bind(inputs: &mut VecDeque<Input>, key_bind: KeyBind) -> KeyBind {
            match inputs.pop_front() {
                Some(input) => {
                    let mut bindings = HashMap::new();
                    bindings.insert(input, create_bind(inputs, key_bind));
                    KeyBind::SubMap(Arc::new(Mutex::new(KeyMap { bindings })))
                },
                None => {
                    key_bind
                }
            }
        }
        match inputs.pop_front() {
            Some(input) => {
                match self.bindings.get(&input) {
                    Some(KeyBind::SubMap(sub_map)) => if !inputs.is_empty() {
                        return sub_map.lock().assign_(inputs, key_bind);
                    },
                    Some(KeyBind::Mapping(mapping)) => if !inputs.is_empty() {
                        for m in mapping.into_iter().rev() {
                            inputs.push_front(*m)
                        }
                        return Err(key_bind);
                    },
                    _ => (),
                }
                self.bindings.insert(input, create_bind(inputs, key_bind));
                Ok(())
            },
            None => unreachable!(),
        }
        //self.bindings.insert(input, binding);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            bindings: HashMap::new(),
        }
    }
}
