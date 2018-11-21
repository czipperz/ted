use std::collections::HashMap;
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
    map: HashMap<Input, KeyBind>,
}

/// The action to be performed when the associated key is pressed.
#[derive(Clone)]
pub enum KeyBind {
    /// Run a function when the key is pressed.
    Action(Arc<fn (&mut State, &mut Display) -> Result<(), ()>>),
    /// Have the user traverse a sub [`KeyMap`] when the key is pressed.
    ///
    /// [`KeyMap`]: struct.KeyMap.html
    SubMap(Arc<Mutex<KeyMap>>),
}

impl KeyMap {
    /// Get the [`KeyBind`] associated with an [`Input`]
    ///
    /// [`Input`]: enum.Input.html
    /// [`KeyBind`]: enum.KeyBind.html
    pub fn lookup(&self, input: &Input) -> Option<&KeyBind> {
        self.map.get(input)
    }

    /// Associate a [`KeyBind`] with an [`Input`]
    ///
    /// [`Input`]: enum.Input.html
    /// [`KeyBind`]: enum.KeyBind.html
    pub fn bind(&mut self, input: Input, bind: KeyBind) {
        self.map.insert(input, bind);
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap { map: HashMap::new() }
    }
}
