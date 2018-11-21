use std::sync::Arc;
use parking_lot::Mutex;
use window::Window;
use layout::Layout;
use key_map::KeyMap;

pub struct State {
    pub windows: Vec<Arc<Mutex<Window>>>,
    pub layout: Layout,
    pub selected_window: Arc<Mutex<Window>>,
    pub default_key_map: Arc<Mutex<KeyMap>>,
}

impl State {
    pub fn new() -> Self {
        let selected_window = Arc::new(Mutex::new(Window::new()));
        State {
            windows: vec![selected_window.clone()],
            layout: Layout::Window(selected_window.clone()),
            selected_window,
            default_key_map: Arc::new(Mutex::new(KeyMap::default())),
        }
    }

    pub fn update_window_cursors(&self) {
        self.layout.update_window_cursors();
    }
}
