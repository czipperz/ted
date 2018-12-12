use command::Command;
use display::Display;
use input::Input;
use key_map::KeyMap;
use mode::Mode;
use parking_lot::Mutex;
use renderer::Renderer;
use std::collections::VecDeque;
use std::sync::Arc;
use window::Window;

/// The state of the program is stored here.
pub struct State {
    pub default_key_map: Arc<Mutex<KeyMap>>,
    pub global_modes: Vec<Arc<Mutex<Mode>>>,
    pub display: Display,
}

impl State {
    /// Create a new `State` with an empty [`Window`].
    ///
    /// [`Window`]: struct.Window.html
    pub fn new<R: 'static + Renderer>(renderer: R) -> Self {
        State {
            default_key_map: Arc::new(Mutex::new(KeyMap::default())),
            global_modes: Vec::new(),
            display: Display::new(Arc::new(Mutex::new(Window::new())), Box::new(renderer)),
        }
    }

    /// This function calls lookup on each
    pub fn lookup(&self, inputs: &mut VecDeque<Input>, throw_away: bool) -> Result<Command, bool> {
        for mode in &self.global_modes {
            match KeyMap::lookup(&mode.lock().key_map, inputs, throw_away) {
                Ok(command) => return Ok(command),
                Err(_) => (),
            }
        }
        KeyMap::lookup(&self.default_key_map, inputs, throw_away)
    }
}
