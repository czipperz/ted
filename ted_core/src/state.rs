use command::Command;
use display::Display;
use input::Input;
use key_map::KeyMap;
use mode::*;
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

    /// This function looks up what [`Command`] an input is bound to.
    ///
    /// First this looks up key bindings on each [`Mode`].  If one is
    /// found, it immediately returns it.  If a mapping is not found
    /// in a [`Mode`], the [`FallthroughBehavior`] is checked.
    ///
    /// For more information on how this function works, look at
    /// [`KeyMap::lookup`].
    ///
    /// [`Command`]: type.Command.html
    /// [`KeyMap::lookup`]: struct.KeyMap.html#method.lookup
    /// [`Mode`]: struct.Mode.html
    pub fn lookup(&self, inputs: &mut VecDeque<Input>, throw_away: bool) -> Result<Arc<Command>, bool> {
        for mode in &self.global_modes {
            let mode = mode.lock();
            let err;
            match KeyMap::lookup(&mode.key_map, inputs, throw_away) {
                Ok(command) => return Ok(command),
                Err(e) => err = e,
            }
            match mode.fallthrough_behavior {
                FallthroughBehavior::Fallthrough => (),
                FallthroughBehavior::Error => return Err(err),
            }
        }
        KeyMap::lookup(&self.default_key_map, inputs, throw_away)
    }
}
