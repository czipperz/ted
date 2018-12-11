use display::Display;
use key_map::KeyMap;
use parking_lot::Mutex;
use renderer::Renderer;
use std::sync::Arc;
use window::Window;

/// The state of the program is stored here.
pub struct State {
    pub default_key_map: Arc<Mutex<KeyMap>>,
    pub display: Display,
}

impl State {
    /// Create a new `State` with an empty [`Window`].
    ///
    /// [`Window`]: struct.Window.html
    pub fn new<R: 'static + Renderer>(renderer: R) -> Self {
        State {
            default_key_map: Arc::new(Mutex::new(KeyMap::default())),
            display: Display::new(Arc::new(Mutex::new(Window::new())), Box::new(renderer)),
        }
    }
}
