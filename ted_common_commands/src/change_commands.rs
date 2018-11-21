use ted_core::*;

pub fn undo_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    let selected_window = state.selected_window.lock();
    let mut buffer = selected_window.buffer.lock();
    buffer.undo();
    Ok(())
}

pub fn redo_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    let selected_window = state.selected_window.lock();
    let mut buffer = selected_window.buffer.lock();
    buffer.redo();
    Ok(())
}
