use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;

/// Split the selected [`Window`](../ted_core/struct.Window.html) in two vertically -- that is into a left and right part.
pub fn vertical_split_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    fn vertical_split_command_(layout: &mut Layout, window: &Arc<Mutex<Window>>) {
        let split = match layout {
            Layout::Window(w) =>
                if Arc::ptr_eq(w, window) {
                    Some(w.clone())
                } else {
                    None
                },
            Layout::VSplit { left, right } => {
                vertical_split_command_(left, window);
                vertical_split_command_(right, window);
                None
            },
            Layout::HSplit { top, bottom } => {
                vertical_split_command_(top, window);
                vertical_split_command_(bottom, window);
                None
            },
        };
        match split {
            Some(w) => {
                let right = Box::new(Layout::Window(clone_window(window)));
                *layout = Layout::VSplit {
                    left: Box::new(Layout::Window(w)),
                    right,
                };
            },
            None => {},
        }
    }
    vertical_split_command_(&mut state.layout, &state.selected_window);
    Ok(())
}

/// Split the selected [`Window`](../ted_core/struct.Window.html) in two horizontally -- that is into a top and bottom part.
pub fn horizontal_split_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    fn horizontal_split_command_(layout: &mut Layout, window: &Arc<Mutex<Window>>) {
        let split = match layout {
            Layout::Window(w) =>
                if Arc::ptr_eq(w, window) {
                    Some(w.clone())
                } else {
                    None
                },
            Layout::VSplit { left, right } => {
                horizontal_split_command_(left, window);
                horizontal_split_command_(right, window);
                None
            },
            Layout::HSplit { top, bottom } => {
                horizontal_split_command_(top, window);
                horizontal_split_command_(bottom, window);
                None
            },
        };
        match split {
            Some(w) => {
                let bottom = Box::new(Layout::Window(clone_window(window)));;
                *layout = Layout::HSplit {
                    top: Box::new(Layout::Window(w)),
                    bottom,
                };
            },
            None => {},
        }
    }
    horizontal_split_command_(&mut state.layout, &state.selected_window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_split_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());
        vertical_split_command(&mut state, &mut display).unwrap();
        match state.layout {
            Layout::VSplit { left, right } => {
                match (*left, *right) {
                    (Layout::Window(left), Layout::Window(right)) => {
                        assert!(!Arc::ptr_eq(&left, &right));
                        let left = left.lock();
                        let right = right.lock();
                        assert!(Arc::ptr_eq(&left.buffer, &right.buffer));
                        assert!(Arc::ptr_eq(&left.buffer_key_map, &right.buffer_key_map));
                        assert_eq!(left.cursor, right.cursor);
                    },
                    _ => panic!(),
                }
            },
            _ => panic!(),
        }
    }

    #[test]
    fn horizontal_split_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());
        horizontal_split_command(&mut state, &mut display).unwrap();
        match &state.layout {
            &Layout::HSplit { ref top, ref bottom } => {
                match (&**top, &**bottom) {
                    (&Layout::Window(ref top), &Layout::Window(ref bottom)) => {
                        assert!(!Arc::ptr_eq(&top, &bottom));
                        let top = top.lock();
                        let bottom = bottom.lock();
                        assert!(Arc::ptr_eq(&top.buffer, &bottom.buffer));
                        assert!(Arc::ptr_eq(&top.buffer_key_map, &bottom.buffer_key_map));
                        assert_eq!(top.cursor, bottom.cursor);
                    },
                    _ => panic!(),
                }
            },
            _ => panic!(),
        }

        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);

        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
    }
}
