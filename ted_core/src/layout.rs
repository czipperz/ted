use std::sync::Arc;
use parking_lot::Mutex;
use window::Window;

#[derive(Clone)]
pub enum Layout {
    Window(Arc<Mutex<Window>>),
    VSplit { left: Box<Layout>, right: Box<Layout> },
    HSplit { top: Box<Layout>, bottom: Box<Layout> },
}

impl Layout {
    pub fn update_window_cursors(&self) {
        match self {
            Layout::Window(window) => {
                let mut window = window.lock();
                window.update_cursor();
            },
            Layout::VSplit { left, right } => {
                left.update_window_cursors();
                right.update_window_cursors();
            },
            Layout::HSplit { top, bottom } => {
                top.update_window_cursors();
                bottom.update_window_cursors();
            },
        }
    }
}

use std::fmt;
impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Layout::Window(_) =>
                write!(f, "Window(..)"),
            Layout::VSplit { left, right } =>
                write!(f, "VSplit {{ left: {:?}, right: {:?} }}", left, right),
            Layout::HSplit { top, bottom } =>
                write!(f, "HSplit {{ top: {:?}, bottom: {:?} }}", top, bottom),
        }
    }
}
