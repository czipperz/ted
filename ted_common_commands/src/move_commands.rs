use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

/// Move backwards one char on the selected [`Window`](../ted_core/struct.Window.html).
pub fn begin_of_buffer_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    selected_window.cursor.set(&buffer, 0);
    Ok(())
}

/// Move forwards one char on the selected [`Window`](../ted_core/struct.Window.html).
pub fn end_of_buffer_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    selected_window.cursor.set(&buffer, buffer.len());
    Ok(())
}

/// Move backwards one char on the selected [`Window`](../ted_core/struct.Window.html).
pub fn backward_char_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    selected_window.increment_cursor(-1);
    Ok(())
}

/// Move forwards one char on the selected [`Window`](../ted_core/struct.Window.html).
pub fn forward_char_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    selected_window.increment_cursor(1);
    Ok(())
}

/// Move to the beginning of the line the selected [`Window`](../ted_core/struct.Window.html)'s cursor is on.
pub fn begin_of_line_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = begin_of_line(&buffer, selected_window.cursor.get());
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Get the point representing the beginning of the line `location` is on.
pub fn begin_of_line(buffer: &Buffer, mut location: usize) -> usize {
    loop {
        if location == 0 {
            break;
        }
        location -= 1;
        if buffer.get(location).unwrap() == '\n' {
            location += 1;
            break;
        }
    }
    location
}

/// Move to the end of the line the selected [`Window`](../ted_core/struct.Window.html)'s cursor is on.
pub fn end_of_line_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = end_of_line(&buffer, selected_window.cursor.get());
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Get the point representing the end of the line `location` is on.
pub fn end_of_line(buffer: &Buffer, mut location: usize) -> usize {
    let buffer_len = buffer.len();
    loop {
        if location == buffer_len {
            break;
        }
        if buffer.get(location).unwrap() == '\n' {
            break;
        }
        location += 1;
    }
    location
}

/// Move forward one line in the selected [`Window`](../ted_core/struct.Window.html).
pub fn forward_line_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = forward_line(&buffer, selected_window.cursor.get(), 1);
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Move backward one line in the selected [`Window`](../ted_core/struct.Window.html).
pub fn backward_line_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = forward_line(&buffer, selected_window.cursor.get(), -1);
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Get the point `times` lines after `location`.
pub fn forward_line(buffer: &Buffer, location: usize, times: isize) -> usize {
    let begin = begin_of_line(buffer, location);
    let mut new_location = begin;
    let offset = location - begin;
    let buffer_len = buffer.len();
    if times < 0 {
        for _ in times..0 {
            if new_location == 0 {
                return std::cmp::min(new_location + offset, end_of_line(buffer, new_location));
            }
            new_location = begin_of_line(buffer, new_location - 1);
        }
    } else {
        for _ in 0..times {
            new_location = end_of_line(buffer, new_location);
            if new_location == buffer_len {
                return begin_of_line(buffer, new_location) + offset;
            }
            new_location += 1;
        }
    }
    new_location = std::cmp::min(new_location + offset, end_of_line(buffer, new_location));
    new_location
}

/// Move forward to the ending of the word in the selected [`Window`](../ted_core/struct.Window.html).
pub fn forward_word_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = forward_word(&buffer, selected_window.cursor.get(), 1);
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Move backward to the beginning of the word in the selected [`Window`](../ted_core/struct.Window.html).
pub fn backward_word_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_window = state.lock().display.selected_window();
    let mut selected_window = selected_window.lock();
    let selected_window = &mut *selected_window;
    let buffer = selected_window.buffer.lock();
    let new_location = forward_word(&buffer, selected_window.cursor.get(), -1);
    selected_window.cursor.set(&buffer, new_location);
    Ok(())
}

/// Get the point `times` words after `location`.
pub fn forward_word(buffer: &Buffer, mut location: usize, times: isize) -> usize {
    fn word_character(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    if times < 0 {
        for _ in times..0 {
            if location == 0 {
                return location;
            }
            location -= 1;
            while !word_character(buffer.get(location).unwrap()) {
                if location == 0 {
                    return location;
                }
                location -= 1;
            }
            while word_character(buffer.get(location).unwrap()) {
                if location == 0 {
                    return location;
                }
                location -= 1;
            }
            location += 1;
        }
    } else {
        let buffer_len = buffer.len();
        for _ in 0..times {
            if location + 1 >= buffer_len {
                return buffer_len;
            }
            while !word_character(buffer.get(location).unwrap()) {
                if location + 1 >= buffer_len {
                    return buffer_len;
                }
                location += 1;
            }
            while word_character(buffer.get(location).unwrap()) {
                if location + 1 >= buffer_len {
                    return buffer_len;
                }
                location += 1;
            }
        }
    }
    location
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_of_buffer_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        begin_of_buffer_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);

            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd\nefgh\nijkl\n").unwrap();
            selected_window.cursor.set(&buffer, 4);
        }

        begin_of_buffer_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
        }
    }

    #[test]
    fn end_of_buffer_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        end_of_buffer_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);

            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd\nefgh\nijkl\n").unwrap();
            selected_window.cursor.set(&buffer, 4);
        }

        end_of_buffer_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 15);
        }
    }

    #[test]
    fn begin_of_line_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        begin_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd\nefgh\nijkl\n").unwrap();
            selected_window.cursor.set(&buffer, 4);
        }

        begin_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
            selected_window.set_cursor(7);
        }

        begin_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 5);
            selected_window.set_cursor(1000);
        }

        begin_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 15);
        }
    }

    #[test]
    fn end_of_line_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            assert_eq!(selected_window.cursor.get(), 0);
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd\nefgh\nijkl\n").unwrap();
            selected_window.cursor.set(&buffer, 4);
        }

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            selected_window.set_cursor(7);
        }

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 9);
        }

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 9);
            selected_window.set_cursor(0);
        }

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            selected_window.set_cursor(1000);
        }

        end_of_line_command(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 15);
        }
    }

    #[test]
    fn forward_line_empty_buffer() {
        let buffer = Buffer::new("*scratch*".into());
        assert_eq!(forward_line(&buffer, 0, 1), 0);
        assert_eq!(forward_line(&buffer, 0, -1), 0);
    }

    #[test]
    fn forward_line_two_lines_same_len() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "Hello\nWorld").unwrap();
        assert_eq!(forward_line(&buffer, 0, 1), 6);
        assert_eq!(forward_line(&buffer, 2, 1), 8);
        assert_eq!(forward_line(&buffer, 8, 1), 8);

        assert_eq!(forward_line(&buffer, 0, -1), 0);
        assert_eq!(forward_line(&buffer, 2, -1), 2);
        assert_eq!(forward_line(&buffer, 6, -1), 0);
        assert_eq!(forward_line(&buffer, 8, -1), 2);
    }

    #[test]
    fn forward_line_two_lines_bigger_first() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "Hello\nW").unwrap();
        assert_eq!(forward_line(&buffer, 0, 1), 6);
        assert_eq!(forward_line(&buffer, 2, 1), 7);
        assert_eq!(forward_line(&buffer, 7, 1), 7);

        assert_eq!(forward_line(&buffer, 0, -1), 0);
        assert_eq!(forward_line(&buffer, 2, -1), 2);
        assert_eq!(forward_line(&buffer, 6, -1), 0);
        assert_eq!(forward_line(&buffer, 7, -1), 1);
    }

    #[test]
    fn forward_line_two_lines_bigger_second() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "H\nWorld").unwrap();
        assert_eq!(forward_line(&buffer, 0, 1), 2);
        assert_eq!(forward_line(&buffer, 1, 1), 3);
        assert_eq!(forward_line(&buffer, 3, 1), 3);

        assert_eq!(forward_line(&buffer, 0, -1), 0);
        assert_eq!(forward_line(&buffer, 1, -1), 1);
        assert_eq!(forward_line(&buffer, 3, -1), 1);
        assert_eq!(forward_line(&buffer, 5, -1), 1);
    }

    #[test]
    fn forward_line_multiple_lines() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "H\n").unwrap();
        buffer.insert_str(2, "ello\n").unwrap();
        buffer.insert_str(7, "\n").unwrap();
        buffer.insert_str(8, "World").unwrap();

        assert_eq!(forward_line(&buffer, 4, -3), 1);
        assert_eq!(forward_line(&buffer, 7, -3), 0);
        assert_eq!(forward_line(&buffer, 9, -3), 1);
        assert_eq!(forward_line(&buffer, 11, -2), 5);

        assert_eq!(forward_line(&buffer, 1, 2), 7);
        assert_eq!(forward_line(&buffer, 4, 3), 10);
        assert_eq!(forward_line(&buffer, 7, 3), 8);
        assert_eq!(forward_line(&buffer, 9, 3), 9);

        assert_eq!(forward_line(&buffer, 4, 0), 4);
        assert_eq!(forward_line(&buffer, 11, 0), 11);
    }

    #[test]
    fn forward_word_empty_buffer() {
        let buffer = Buffer::new("*scratch*".into());
        assert_eq!(forward_word(&buffer, 0, 1), 0);
        assert_eq!(forward_word(&buffer, 0, 0), 0);
        assert_eq!(forward_word(&buffer, 0, -1), 0);
    }

    #[test]
    fn forward_word_multiple_lines() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "Hello\n").unwrap();
        buffer.insert_str(6, "\n").unwrap();
        buffer.insert_str(7, "World").unwrap();
        assert_eq!(forward_word(&buffer, 0, 1), 5);
        assert_eq!(forward_word(&buffer, 3, 1), 5);
        assert_eq!(forward_word(&buffer, 4, 1), 5);
        assert_eq!(forward_word(&buffer, 5, 1), 12);
        assert_eq!(forward_word(&buffer, 3, 2), 12);
        assert_eq!(forward_word(&buffer, 3, -1), 0);
        assert_eq!(forward_word(&buffer, 7, -1), 0);
        assert_eq!(forward_word(&buffer, 8, -1), 7);
    }

    #[test]
    fn forward_word_multiple_lines_spaces() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " Hello \n").unwrap();
        buffer.insert_str(8, " \n").unwrap();
        buffer.insert_str(10, " World ").unwrap();
        assert_eq!(forward_word(&buffer, 0, 1), 6);
        assert_eq!(forward_word(&buffer, 4, 1), 6);
        assert_eq!(forward_word(&buffer, 5, 1), 6);
        assert_eq!(forward_word(&buffer, 6, 1), 16);
        assert_eq!(forward_word(&buffer, 3, 2), 16);
        assert_eq!(forward_word(&buffer, 3, 3), 17);

        assert_eq!(forward_word(&buffer, 1, -1), 0);
        assert_eq!(forward_word(&buffer, 7, -1), 1);
        assert_eq!(forward_word(&buffer, 8, -1), 1);
        assert_eq!(forward_word(&buffer, 12, -1), 11);
    }

    #[test]
    fn forward_word_underscore() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "a_b").unwrap();
        assert_eq!(forward_word(&buffer, 0, 1), 3);
    }
}
