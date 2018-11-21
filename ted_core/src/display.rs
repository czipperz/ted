use state::State;
use input::Input;

/// A generic interface to handle a graphical Display.
///
/// Specific implementations include [`DebugDisplay`] and [`CursesDisplay`].
///
/// [`DebugDisplay`]: struct.DebugDisplay.html
/// [`CursesDisplay`]: struct.CursesDisplay.html
pub trait Display {
    /// Show the [`State`] on the `Display`.
    ///
    /// [`State`]: struct.State.html
    fn show(&mut self, state: &State) -> Result<(), ()>;

    /// Get the next user [`Input`] event if any.
    ///
    /// [`Input`]: enum.Input.html
    fn getch(&mut self) -> Option<Input>;
}
