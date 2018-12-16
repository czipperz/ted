use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

/// Delete backwards one char in the selected [`Window`](../ted_core/struct.Window.html).
#[derive(Debug)]
pub struct DeleteBackwardCharCommand;

/// Construct a [`DeleteBackwardCharCommand`].
///
/// [`DeleteBackwardCharCommand`]: struct.DeleteBackwardCharCommand.html
pub fn delete_backward_char_command() -> Arc<DeleteBackwardCharCommand> {
    Arc::new(DeleteBackwardCharCommand)
}

impl Command for DeleteBackwardCharCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        let selected_window = &mut *selected_window;
        if selected_window.cursor.get() != 0 {
            let mut buffer = selected_window.buffer.lock();
            debug_assert!(selected_window.cursor.is_updated(&buffer));
            buffer.delete(selected_window.cursor.get() - 1).unwrap();
            selected_window.cursor.update(&buffer);
        }
        Ok(())
    }
}

/// Delete forwards one char in the selected [`Window`](../ted_core/struct.Window.html).
#[derive(Debug)]
pub struct DeleteForwardCharCommand;

/// Construct a [`DeleteForwardCharCommand`].
///
/// [`DeleteForwardCharCommand`]: struct.DeleteForwardCharCommand.html
pub fn delete_forward_char_command() -> Arc<DeleteForwardCharCommand> {
    Arc::new(DeleteForwardCharCommand)
}

impl Command for DeleteForwardCharCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        let selected_window = &mut *selected_window;
        let mut buffer = selected_window.buffer.lock();
        debug_assert!(selected_window.cursor.is_updated(&buffer));
        if selected_window.cursor.get() != buffer.len() {
            buffer.delete(selected_window.cursor.get()).unwrap();
            selected_window.cursor.update(&buffer);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_backward_char_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let buffer = buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
        }

        delete_backward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);

            let mut buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
            buffer.insert_str(0, "abcd").unwrap();
            assert_eq!(format!("{}", *buffer), "abcd");
        }

        delete_backward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcd");

            selected_window.cursor.set(&buffer, 2);
        }

        delete_backward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);
            selected_window.set_cursor(3);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "acd");
        }

        delete_backward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ac");
        }
    }

    #[test]
    fn delete_forward_char_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let buffer = buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
        }

        delete_forward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);

            {
                let mut buffer = selected_window.buffer.lock();
                assert_eq!(format!("{}", *buffer), "");
                buffer.insert_str(0, "abcd").unwrap();
                assert_eq!(format!("{}", *buffer), "abcd");
            }

            selected_window.set_cursor(4);
        }

        delete_forward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 4);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcd");

            selected_window.cursor.set(&buffer, 2);
        }

        delete_forward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            selected_window.set_cursor(0);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abd");
        }

        delete_forward_char_command()
            .execute(state.clone())
            .unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "bd");
        }
    }
}
