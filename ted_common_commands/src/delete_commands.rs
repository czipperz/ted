use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;

/// Delete backwards one char in the selected [`Window`](../ted_core/struct.Window.html).
pub fn delete_backward_char_command(state: Arc<Mutex<State>>,
                                    _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    if selected_window.cursor.get() != 0 {
        let mut buffer = selected_window.buffer.lock();
        buffer.delete(selected_window.cursor.get() - 1).unwrap();
        selected_window.cursor.update(&buffer);
    }
    Ok(())
}

/// Delete forwards one char in the selected [`Window`](../ted_core/struct.Window.html).
pub fn delete_forward_char_command(state: Arc<Mutex<State>>,
                                   _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let mut buffer = selected_window.buffer.lock();
    if selected_window.cursor.get() != buffer.len() {
        buffer.delete(selected_window.cursor.get()).unwrap();
        selected_window.cursor.update(&buffer);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_backward_char_command_1() {
        let state = Arc::new(Mutex::new(State::new()));
        let display = Arc::new(Mutex::new(DebugDisplay::new(Vec::new())));

        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
        }

        delete_backward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            let mut buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
            buffer.insert_str(0, "abcd").unwrap();
            assert_eq!(format!("{}", *buffer), "abcd");
        }

        delete_backward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            let selected_window = &mut *selected_window;
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcd");

            selected_window.cursor.set(&buffer, 2);
        }

        delete_backward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);
            selected_window.set_cursor(3);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "acd");
        }

        delete_backward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ac");
        }
    }

    #[test]
    fn delete_forward_char_command_1() {
        let state = Arc::new(Mutex::new(State::new()));
        let display = Arc::new(Mutex::new(DebugDisplay::new(Vec::new())));

        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
        }

        delete_forward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            let selected_window = &mut *selected_window;
            let mut buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
            buffer.insert_str(0, "abcd").unwrap();
            assert_eq!(format!("{}", *buffer), "abcd");
            selected_window.cursor.set(&buffer, 4);
        }

        delete_forward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            let selected_window = &mut *selected_window;
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcd");

            selected_window.cursor.set(&buffer, 2);
        }

        delete_forward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            selected_window.set_cursor(0);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abd");
        }

        delete_forward_char_command(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "bd");
        }
    }
}
