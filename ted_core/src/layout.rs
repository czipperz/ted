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
    /// Unwrap this as if it is the enumeration type `Window`
    ///
    /// If this is not a `Window` then panic.
    pub fn unwrap_window(&self) -> &Arc<Mutex<Window>> {
        match self {
            Layout::Window(window) => window,
            _ => panic!("`Layout::unwrap_window` called not on a `Layout::Window`"),
        }
    }

    /// Replace the `selected_window` with a new layout.
    pub fn replace_selected_window<L: Into<Layout>>(
        &mut self,
        selected_window: &Arc<Mutex<Window>>,
        new_layout: L,
    ) {
        fn replace_selected_window(
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
                    return replace_selected_window(left, selected_window, new_layout).and_then(
                        |new_layout| replace_selected_window(right, selected_window, new_layout),
                    );
                }
                Layout::HSplit { top, bottom } => {
                    return replace_selected_window(top, selected_window, new_layout).and_then(
                        |new_layout| replace_selected_window(bottom, selected_window, new_layout),
                    );
                }
            }
            *layout = new_layout;
            None
        }
        replace_selected_window(self, selected_window, new_layout.into());
    }

    pub fn first_window(&self) -> &Arc<Mutex<Window>> {
        match self {
            Layout::Window(w) => w,
            Layout::VSplit { left, .. } => left.first_window(),
            Layout::HSplit { top, .. } => top.first_window(),
        }
    }

    pub fn last_window(&self) -> &Arc<Mutex<Window>> {
        match self {
            Layout::Window(w) => w,
            Layout::VSplit { right, .. } => right.last_window(),
            Layout::HSplit { bottom, .. } => bottom.last_window(),
        }
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

impl Default for Layout {
    fn default() -> Self {
        Layout::Window(Arc::default())
    }
}

impl From<Arc<Mutex<Window>>> for Layout {
    fn from(w: Arc<Mutex<Window>>) -> Self {
        Layout::Window(w)
    }
}

impl From<Window> for Layout {
    fn from(w: Window) -> Self {
        Arc::new(Mutex::new(w)).into()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_window_vsplit_of_hsplits() {
        let correct: Arc<Mutex<Window>> = Arc::default();
        let layout = Layout::VSplit {
            left: Box::new(Layout::HSplit {
                top: Box::new(Layout::Window(correct.clone())),
                bottom: Box::new(Layout::default()),
            }),
            right: Box::new(Layout::HSplit {
                top: Box::new(Layout::default()),
                bottom: Box::new(Layout::default()),
            }),
        };
        assert!(Arc::ptr_eq(layout.first_window(), &correct));
    }

    #[test]
    fn first_window_hsplit_of_vsplits() {
        let correct: Arc<Mutex<Window>> = Arc::default();
        let layout = Layout::HSplit {
            top: Box::new(Layout::VSplit {
                left: Box::new(Layout::Window(correct.clone())),
                right: Box::new(Layout::default()),
            }),
            bottom: Box::new(Layout::VSplit {
                left: Box::new(Layout::default()),
                right: Box::new(Layout::default()),
            }),
        };
        assert!(Arc::ptr_eq(layout.first_window(), &correct));
    }

    #[test]
    fn last_window_vsplit_of_hsplits() {
        let correct: Arc<Mutex<Window>> = Arc::default();
        let layout = Layout::VSplit {
            left: Box::new(Layout::HSplit {
                top: Box::new(Layout::default()),
                bottom: Box::new(Layout::default()),
            }),
            right: Box::new(Layout::HSplit {
                top: Box::new(Layout::default()),
                bottom: Box::new(Layout::Window(correct.clone())),
            }),
        };
        assert!(Arc::ptr_eq(layout.last_window(), &correct));
    }

    #[test]
    fn last_window_hsplit_of_vsplits() {
        let correct: Arc<Mutex<Window>> = Arc::default();
        let layout = Layout::HSplit {
            top: Box::new(Layout::VSplit {
                left: Box::new(Layout::default()),
                right: Box::new(Layout::default()),
            }),
            bottom: Box::new(Layout::VSplit {
                left: Box::new(Layout::default()),
                right: Box::new(Layout::Window(correct.clone())),
            }),
        };
        assert!(Arc::ptr_eq(layout.last_window(), &correct));
    }
}
