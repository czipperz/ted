use parking_lot::Mutex;
use state::State;
use std::fmt::Debug;
use std::sync::Arc;

/// A command to be ran when a certain sequence of keys are pressed.
///
/// A command must allow account for the fact that multiple commands
/// can potentially run at once.  Thus, if the command is only to edit
/// one [`Buffer`], it should not keep the entire [`State`] locked.  It
/// should lock the [`State`] only to retrieve the [`Buffer`] to edit.
/// Then the [`State`] should be unlocked and just the [`Buffer`] should
/// be locked.  But it should only be locked when being updated.  Thus
/// work can be done by multiple workers at the same time.
///
/// # Examples
///
/// Template for a command that edits the current window:
/// ```
/// extern crate parking_lot;
/// extern crate ted_core;
///
/// # fn main() {
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use ted_core::State;
/// pub fn my_custom_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
///     let selected_window = state.lock().display.selected_window().clone();
///     let mut selected_window = selected_window.lock();
///     // use selected_window
///     Ok(())
/// }
/// # }
/// ```
///
/// Template for a command that edits the current buffer:
/// ```
/// extern crate parking_lot;
/// extern crate ted_core;
///
/// # fn main() {
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use ted_core::State;
/// pub fn my_custom_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
///     let buffer = state.lock().display.selected_window_buffer();
///     let mut buffer = buffer.lock();
///     // use selected_window
///     Ok(())
/// }
/// # }
/// ```
///
/// If the event is moderately short, then the [`Display`] can be safely ignored.
///
/// [`Buffer`]: struct.Buffer.html
/// [`Display`]: trait.Display.html
/// [`State`]: struct.State.html
pub trait Command: Debug + Send + Sync {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String>;
}

#[derive(Debug)]
pub struct BlankCommand;
pub fn blank_command() -> Arc<Command> {
    Arc::new(BlankCommand)
}
impl Command for BlankCommand {
    fn execute(&self, _: Arc<Mutex<State>>) -> Result<(), String> {
        Ok(())
    }
}
