use parking_lot::Mutex;
use std::sync::Arc;
use state::State;

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
/// Template for a command that only edits the current buffer:
/// ```
/// # use ted_core::{Buffer, BufferName};
/// let mut buffer = Buffer::new("*scratch*".into());
/// buffer.insert_str(0, "abcd");
/// buffer.delete_region(2, 4).unwrap();
/// assert_eq!(buffer.len(), 2);
/// assert_eq!(format!("{}", buffer), "ab");
/// ```
/// ```
/// # extern crate parking_lot;
/// # extern crate ted_core;
/// # fn main() {
/// # use std::sync::Arc;
/// # use parking_lot::Mutex;
/// # use ted_core::State;
/// pub fn my_custom_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
///     let selected_window = state.lock().display.selected_window().clone();
///     let mut selected_window = selected_window.lock();
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
pub type Command = Arc<fn(Arc<Mutex<State>>) -> Result<(), ()>>;
