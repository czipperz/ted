use buffer::Buffer;
use input::Input;
use layout::Layout;
use parking_lot::Mutex;
use renderer::Renderer;
use std::sync::Arc;
use window::Window;

pub struct Frame {
    pub layout: Layout,
    pub renderer: Box<Renderer>,
    pub selected_window: Arc<Mutex<Window>>,
}

impl Frame {
    pub fn new(selected_window: Arc<Mutex<Window>>, renderer: Box<Renderer>) -> Self {
        Frame {
            layout: Layout::Window(selected_window.clone()),
            renderer,
            selected_window: selected_window,
        }
    }

    pub fn selected_window_buffer(&self) -> Arc<Mutex<Buffer>> {
        let selected_window = self.selected_window.lock();
        selected_window.buffer.clone()
    }

    pub fn show(&mut self, selected_window: &Arc<Mutex<Window>>) -> Result<(), ()> {
        self.renderer.show(&self.layout, selected_window)
    }

    pub fn getch(&mut self) -> Option<Input> {
        self.renderer.getch()
    }
}
