use buffer::Buffer;
use input::Input;
use layout::Layout;
use parking_lot::Mutex;
use renderer::Renderer;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use window::Window;

pub struct Frame {
    pub layout: Layout,
    pub renderer: Box<Renderer>,
    pub selected_window: Arc<Mutex<Window>>,
    messages: VecDeque<String>,
    message_display_time: Option<Instant>,
}

impl Frame {
    pub fn new(selected_window: Arc<Mutex<Window>>, renderer: Box<Renderer>) -> Self {
        Frame {
            layout: Layout::Window(selected_window.clone()),
            renderer,
            selected_window: selected_window,
            messages: VecDeque::new(),
            message_display_time: None,
        }
    }

    pub fn selected_window_buffer(&self) -> Arc<Mutex<Buffer>> {
        let selected_window = self.selected_window.lock();
        selected_window.buffer.clone()
    }

    pub fn replace_selected_window(&mut self, window: Arc<Mutex<Window>>) {
        self.layout
            .replace_window(&self.selected_window, Layout::Window(window.clone()));
        self.selected_window = window;
    }

    pub fn show(&mut self, is_selected_frame: bool) -> Result<(), String> {
        let selected_window = if is_selected_frame {
            Some(&self.selected_window)
        } else {
            None
        };
        self.renderer.show(
            &self.layout,
            selected_window,
            &mut self.messages,
            &mut self.message_display_time,
        )
    }

    pub fn getch(&mut self) -> Option<Input> {
        self.renderer.getch()
    }

    pub fn add_message<S>(&mut self, message: S)
    where
        S: ToString,
    {
        self.messages.push_back(message.to_string());
    }
}
