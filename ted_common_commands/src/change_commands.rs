use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

/// Undo the last change to the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::undo`].
///
/// [`Buffer::undo`]: ../ted_core/struct.Buffer.html#method.undo
#[derive(Debug)]
pub struct UndoCommand;

/// Construct a [`UndoCommand`].
///
/// [`UndoCommand`]: struct.UndoCommand.html
pub fn undo_command() -> Arc<UndoCommand> {
    Arc::new(UndoCommand)
}

impl Command for UndoCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            selected_window.buffer.clone()
        };
        let mut buffer = buffer.lock();
        buffer.undo()?;
        Ok(())
    }
}

/// Revert the last call to undo on the Buffer of the selected [`Window`](../ted_core/struct.Window.html).
///
/// See [`Buffer::redo`].
///
/// [`Buffer::redo`]: ../ted_core/struct.Buffer.html#method.redo
#[derive(Debug)]
pub struct RedoCommand;

/// Construct a [`RedoCommand`].
///
/// [`RedoCommand`]: struct.RedoCommand.html
pub fn redo_command() -> Arc<RedoCommand> {
    Arc::new(RedoCommand)
}

impl Command for RedoCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            selected_window.buffer.clone()
        };
        let mut buffer = buffer.lock();
        buffer.redo()?;
        Ok(())
    }
}
