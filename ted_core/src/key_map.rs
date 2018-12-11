use command::Command;
use input::Input;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Map [`Input`]s to [`KeyBind`]s.
///
/// [`Input`]: enum.Input.html
/// [`KeyBind`]: enum.KeyBind.html
pub struct KeyMap {
    bindings: HashMap<Input, KeyBind>,
}

/// The command to be performed when the associated key is pressed.
#[derive(Clone)]
enum KeyBind {
    /// Run a function when the key is pressed.
    Command(Command),
    /// Traverse a sub [`KeyMap`] when the key is pressed.
    ///
    /// [`KeyMap`]: struct.KeyMap.html
    SubMap(Arc<Mutex<KeyMap>>),
    /// This key is bound to another sequence of keys.
    Mapping(Vec<Input>),
}

impl KeyMap {
    /// This function performs the lookup.
    ///
    /// `Ok(command)` means an command was found and the inputs leading to
    /// it have been popped from the front of the `VecDeque`.
    ///
    /// `Err(true)` means an command was not found because not enough
    /// user input has been provided.  IE the user hits `C-x` and we
    /// stall for the next key (`C-c` for stop for instance).  We don't
    /// want to throw away those keys so `lookup_` will re-push the
    /// popped keys onto the beginning of the `VecDeque`.
    ///
    /// `Err(false)` means an command was not found because that key
    /// combination doesn't exist.
    pub fn lookup(
        key_map: &Arc<Mutex<KeyMap>>,
        inputs: &mut VecDeque<Input>,
        throw_away: bool,
    ) -> Result<Command, bool> {
        loop {
            match KeyMap::lookup_(key_map, inputs, throw_away)? {
                Ok(command) => return Ok(command),
                // there was a KeyBind::Mapping encountered and we must restart the search
                Err(()) => (),
            }
        }
    }

    /// Bind a sequence of [`Input`]s to run a certain [`Command`].
    ///
    /// [`Input`]: enum.Input.html
    /// [`Command`]: type.Command.html
    pub fn bind(&mut self, inputs: Vec<Input>, command: Command) {
        self.assign(&mut inputs.into(), KeyBind::Command(command))
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

    fn lookup_(
        key_map: &Arc<Mutex<KeyMap>>,
        inputs: &mut VecDeque<Input>,
        throw_away: bool,
    ) -> Result<Result<Command, ()>, bool> {
        match inputs.pop_front() {
            Some(input) => {
                let key_map = key_map.lock();
                match key_map.bindings.get(&input) {
                    Some(KeyBind::Command(command)) => Ok(Ok(command.clone())),
                    Some(KeyBind::SubMap(sub_map)) => KeyMap::lookup_(sub_map, inputs, throw_away)
                        .map_err(|e| {
                            if !throw_away || e {
                                inputs.push_front(input);
                            }
                            e
                        }),
                    Some(KeyBind::Mapping(mapping)) => {
                        for m in mapping.into_iter().rev() {
                            inputs.push_front(*m)
                        }
                        Ok(Err(()))
                    }
                    None => Err(false),
                }
            }
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
                }
                None => key_bind,
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
            }
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
