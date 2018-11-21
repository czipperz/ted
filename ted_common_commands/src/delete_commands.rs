use ted_core::*;

/// Delete backwards one char in the selected [`Window`](../ted_core/struct.Window.html).
pub fn backspace_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    let mut selected_window = state.selected_window.lock();
    let selected_window = &mut *selected_window;
    if selected_window.cursor.get() != 0 {
        let mut buffer = selected_window.buffer.lock();
        buffer.delete(selected_window.cursor.get() - 1).unwrap();
        selected_window.cursor.increment(&buffer, -1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backspace_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

        {
            let selected_window = state.selected_window.lock();
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
        }

        backspace_command(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            let mut buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "");
            buffer.insert_str(0, "abcd").unwrap();
        }

        backspace_command(&mut state, &mut display).unwrap();
        {
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            selected_window.set_cursor(2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcd");
        }

        backspace_command(&mut state, &mut display).unwrap();
        {
            let mut selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);
            selected_window.set_cursor(3);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "acd");
        }

        backspace_command(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ac");
        }
    }
}
