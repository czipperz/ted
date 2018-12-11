use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

/// Undo the last change to the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::undo`].
///
/// [`Buffer::undo`]: ../ted_core/struct.Buffer.html#method.undo
pub fn undo_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let buffer = {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        selected_window.buffer.clone()
    };
    let mut buffer = buffer.lock();
    buffer.undo();
    Ok(())
}

/// Revert the last call to undo on the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::redo`].
///
/// [`Buffer::redo`]: ../ted_core/struct.Buffer.html#method.redo
pub fn redo_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let buffer = {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        selected_window.buffer.clone()
    };
    let mut buffer = buffer.lock();
    buffer.redo();
    Ok(())
}
