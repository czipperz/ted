use parking_lot::Mutex;
use std::sync::Arc;
use window::Window;

/// The layout of the window.
///
/// This structure is recursively defined and thus it is easy to split
/// up the screen into pieces.
#[derive(Clone)]
pub enum Layout {
    /// A single window is at this point in the `Layout`.
    Window(Arc<Mutex<Window>>),
    /// The screen is split into two vertical regions: the `left` and `right`.
    VSplit {
        left: Box<Layout>,
        right: Box<Layout>,
    },
    /// The screen is split into two horizontal regions: the `top` and `bottom`.
    HSplit {
        top: Box<Layout>,
        bottom: Box<Layout>,
    },
}

impl Layout {
    pub fn unwrap_window(&self) -> &Arc<Mutex<Window>> {
        match self {
            Layout::Window(window) => window,
            _ => unimplemented!(),
        }
    }

    pub fn set_selected_window(
        &mut self,
        selected_window: &Arc<Mutex<Window>>,
        new_layout: Layout,
    ) {
        fn set_selected_window(
            layout: &mut Layout,
            selected_window: &Arc<Mutex<Window>>,
            new_layout: Layout,
        ) -> Option<Layout> {
            match layout {
                Layout::Window(window) => {
                    if !Arc::ptr_eq(window, selected_window) {
                        return Some(new_layout);
                    }
                }
                Layout::VSplit { left, right } => {
                    return set_selected_window(left, selected_window, new_layout).and_then(
                        |new_layout| set_selected_window(right, selected_window, new_layout),
                    );
                }
                Layout::HSplit { top, bottom } => {
                    return set_selected_window(top, selected_window, new_layout).and_then(
                        |new_layout| set_selected_window(bottom, selected_window, new_layout),
                    );
                }
            }
            *layout = new_layout;
            None
        }
        set_selected_window(self, selected_window, new_layout);
    }

    /// Recursively walk the `Layout` and update all the cursors along the way.
    ///
    /// See [`Window::update_cursor`].
    ///
    /// [`Window::update_cursor`]: struct.Window.html#method.update_cursor
    pub fn update_window_cursors(&self) {
        match self {
            Layout::Window(window) => {
                let mut window = window.lock();
                window.update_cursor();
            }
            Layout::VSplit { left, right } => {
                left.update_window_cursors();
                right.update_window_cursors();
            }
            Layout::HSplit { top, bottom } => {
                top.update_window_cursors();
                bottom.update_window_cursors();
            }
        }
    }
}

use std::fmt;
impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Layout::Window(_) => write!(f, "Window(..)"),
            Layout::VSplit { left, right } => {
                write!(f, "VSplit {{ left: {:?}, right: {:?} }}", left, right)
            }
            Layout::HSplit { top, bottom } => {
                write!(f, "HSplit {{ top: {:?}, bottom: {:?} }}", top, bottom)
            }
        }
    }
}
