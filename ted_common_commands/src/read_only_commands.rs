use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

#[derive(Debug)]
pub struct ToggleReadOnly;

/// Construct a [`ToggleReadOnly`].
///
/// [`ToggleReadOnly`]: struct.ToggleReadOnly.html
pub fn toggle_read_only_command() -> Arc<ToggleReadOnly> {
    Arc::new(ToggleReadOnly)
}

impl Command for ToggleReadOnly {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let buffer = state.lock().display.selected_window_buffer();
        let mut buffer = buffer.lock();
        buffer.read_only = !buffer.read_only;
        Ok(())
    }
}
