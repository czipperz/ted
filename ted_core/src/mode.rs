use key_map::KeyMap;
use parking_lot::Mutex;
use std::sync::Arc;

/// A `Mode` allows a [`KeyMap`] to be applied only part of the time.
///
/// This can either be turned on globally via [`State::global_modes`],
/// [`Window::window_modes`], or [`Buffer::buffer_modes`].
///
/// [`KeyMap`]: struct.KeyMap.html
/// [`State::global_modes`]: struct.State.html#structfield.global_modes
/// [`Window::window_modes`]: struct.Window.html#structfield.window_modes
/// [`Buffer::buffer_modes`]: struct.Buffer.html#structfield.buffer_modes
pub struct Mode {
    /// The mapping
    pub key_map: Arc<Mutex<KeyMap>>,
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
        Mode { key_map }
    }
}
