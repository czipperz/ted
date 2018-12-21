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
    Command(Arc<Command>),
    /// Traverse a sub [`KeyMap`] when the key is pressed.
    ///
    /// [`KeyMap`]: struct.KeyMap.html
    SubMap(Arc<Mutex<KeyMap>>),
    /// This key is bound to another sequence of keys.
    Mapping(Vec<Input>),
}

pub enum LookupError {
    NotEnoughInput,
    UnboundInput(Option<Input>),
    InputWasMapped,
}

impl KeyMap {
    /// This function performs the lookup.
    ///
    /// `Ok(command)` means an command was found and the inputs leading to
    /// it have been popped from the front of the `VecDeque`.
    ///
    /// `Err(LookupError::NotEnoughInput)` means an command was not
    /// found because not enough user input has been provided.  IE the
    /// user hits `C-x` and we stall for the next key (`C-c` for stop
    /// for instance).  We don't want to throw away those keys so
    /// `lookup_` will re-push the popped keys onto the beginning of
    /// the `VecDeque`.
    ///
    /// `Err(LookupError::UnboundInput)` means an command was not
    /// found because that key combination doesn't exist.
    ///
    /// `Err(LookupError::InputWasMapped)` means that the input
    /// sequence is mapped to a different input sequence, and the
    /// parent function should reinvoke this with the same arguments.
    pub fn lookup(
        key_map: &Arc<Mutex<KeyMap>>,
        inputs: &mut VecDeque<Input>,
        save_input_on_error: bool,
    ) -> Result<Arc<Command>, LookupError> {
        match inputs.pop_front() {
            Some(input) => {
                let key_map = key_map.lock();
                match key_map.bindings.get(&input) {
                    Some(KeyBind::Command(command)) => Ok(command.clone()),
                    Some(KeyBind::SubMap(sub_map)) => {
                        KeyMap::lookup(sub_map, inputs, save_input_on_error).map_err(|e| {
                            if save_input_on_error {
                                inputs.push_front(input);
                            }
                            match e {
                                LookupError::NotEnoughInput => {
                                    if !save_input_on_error {
                                        inputs.push_front(input);
                                    }
                                    e
                                }
                                LookupError::UnboundInput(_) => LookupError::UnboundInput(None),
                                LookupError::InputWasMapped => e,
                            }
                        })
                    }
                    Some(KeyBind::Mapping(mapping)) => {
                        for m in mapping.into_iter().rev() {
                            inputs.push_front(*m)
                        }
                        Err(LookupError::InputWasMapped)
                    }
                    None => {
                        if save_input_on_error {
                            inputs.push_front(input);
                        }
                        Err(LookupError::UnboundInput(Some(input)))
                    }
                }
            }
            None => Err(LookupError::NotEnoughInput),
        }
    }

    /// Bind a sequence of [`Input`]s to run a certain [`Command`].
    ///
    /// [`Input`]: enum.Input.html
    /// [`Command`]: type.Command.html
    pub fn bind(&mut self, inputs: Vec<Input>, command: Arc<Command>) {
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
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            bindings: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_map_lookup_bound_key() {
        let mut key_map = KeyMap::default();
        key_map.bind(vec![kbd!('g')], blank_command());
        let key_map = Arc::new(Mutex::new(key_map));
        let mut input = vec![kbd!('g')].into();
        assert!(KeyMap::lookup(&key_map, &mut input, true).is_ok());
        assert_eq!(input, vec![]);
        let mut input = vec![kbd!('g')].into();
        assert!(KeyMap::lookup(&key_map, &mut input, false).is_ok());
        assert_eq!(input, vec![]);
    }

    #[test]
    fn key_map_lookup_unbound_key() {
        let key_map = Arc::new(Mutex::new(KeyMap::default()));
        let mut input = vec![kbd!(C - 'x')].into();
        assert_eq!(
            KeyMap::lookup(&key_map, &mut input, true).unwrap_err(),
            LookupError::UnboundInput(Some(kbd!(C - 'x')))
        );
        assert_eq!(input, vec![kbd!(C - 'x')]);
        let mut input = vec![kbd!(C - 'x')].into();
        assert_eq!(
            KeyMap::lookup(&key_map, &mut input, false).unwrap_err(),
            LookupError::UnboundInput(Some(kbd!(C - 'x')))
        );
        assert_eq!(input, vec![]);
    }
}
