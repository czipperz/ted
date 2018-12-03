use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;

/// Undo the last change to the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::undo`].
///
/// [`Buffer::undo`]: ../ted_core/struct.Buffer.html#method.undo
pub fn undo_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let selected_window = selected_window.lock();
    let mut buffer = selected_window.buffer.lock();
    buffer.undo();
    Ok(())
}

/// Revert the last call to undo on the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::redo`].
///
/// [`Buffer::redo`]: ../ted_core/struct.Buffer.html#method.redo
pub fn redo_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let selected_window = selected_window.lock();
    let mut buffer = selected_window.buffer.lock();
    buffer.redo();
    Ok(())
}
