use input::Input;
use layout::Layout;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use window::Window;

/// A generic interface to render a graphical [`Frame`].
///
/// Specific implementations include [`DebugRenderer`] and [`CursesRenderer`].
///
/// [`DebugFrame`]: struct.DebugFrame.html
/// [`CursesFrame`]: struct.CursesFrame.html
pub trait Renderer: Send {
    /// Show the [`Layout`] on the `Frame`.
    ///
    /// [`Layout`]: enum.Layout.html
    fn show(&mut self, layout: &Layout, selected_window: Option<&Arc<Mutex<Window>>>,
            messages: &mut VecDeque<String>, message_display_time: &mut Option<Instant>) -> Result<(), String>;

    /// Get the next user [`Input`] event if any.
    ///
    /// [`Input`]: enum.Input.html
    fn getch(&mut self) -> Option<Input>;
}
