use buffer::Buffer;
use debug_renderer::*;
use frame::Frame;
use input::Input;
use parking_lot::Mutex;
use parking_lot::{MappedMutexGuard, MutexGuard};
use renderer::Renderer;
use std::sync::Arc;
use window::Window;

pub struct Display {
    pub selected_frame: Arc<Mutex<Frame>>,
    pub frames: Vec<Arc<Mutex<Frame>>>,
}

impl Display {
    pub fn new(selected_window: Arc<Mutex<Window>>, renderer: Box<Renderer>) -> Self {
        let frame = Arc::new(Mutex::new(Frame::new(selected_window, renderer)));
        Display {
            selected_frame: frame.clone(),
            frames: vec![frame],
        }
    }

    pub fn selected_window(&self) -> Arc<Mutex<Window>> {
        self.selected_frame.lock().selected_window.clone()
    }

    pub fn selected_window_buffer(&self) -> Arc<Mutex<Buffer>> {
        self.selected_frame.lock().selected_window_buffer()
    }

    pub unsafe fn debug_renderer(&self) -> MappedMutexGuard<DebugRenderer> {
        MutexGuard::map(self.selected_frame.lock(), |f| {
            debug_renderer(&mut *f.renderer)
        })
    }

    pub fn update_cursors(&self) {
        for frame in &self.frames {
            frame.lock().layout.update_window_cursors();
        }
    }

    pub fn show(&self) -> Result<(), ()> {
        for frame in &self.frames {
            frame.lock().show(Arc::ptr_eq(&self.selected_frame, frame))?
        }
        Ok(())
    }

    pub fn getch(&self) -> Option<Input> {
        for frame in &self.frames {
            match frame.lock().renderer.getch() {
                Some(input) => return Some(input),
                None => (),
            }
        }
        None
    }
}
