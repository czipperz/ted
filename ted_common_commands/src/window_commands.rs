use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;

fn find_first_window(layout: &Layout) -> &Arc<Mutex<Window>> {
    match layout {
        Layout::Window(w) => w,
        Layout::VSplit { left, .. } => find_first_window(left),
        Layout::HSplit { top, .. } => find_first_window(top),
    }
}

fn find_last_window(layout: &Layout) -> &Arc<Mutex<Window>> {
    match layout {
        Layout::Window(w) => w,
        Layout::VSplit { right, .. } => find_first_window(right),
        Layout::HSplit { bottom, .. } => find_first_window(bottom),
    }
}

/// Close the selected [`Window`] and select the next one in a clockwise fashion.
///
/// If the [`Layout`] is only a single [`Window`], then nothing happens.
/// Otherwise, the immediate parent split is simplified to the other [`Layout`].
///
/// [`Window`]:../ted_core/struct.Window.html
/// [`Layout`]:../ted_core/enum.Layout.html
pub fn close_window_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    fn close_window_command_(layout: &mut Layout, window: &mut Arc<Mutex<Window>>) -> bool {
        let new_layout = match layout {
            Layout::Window(w) => return Arc::ptr_eq(w, window),
            Layout::VSplit { left, right } => {
                if close_window_command_(left, window) {
                    Some(right.clone())
                } else if close_window_command_(right, window) {
                    Some(left.clone())
                } else {
                    None
                }
            },
            Layout::HSplit { top, bottom } => {
                if close_window_command_(top, window) {
                    Some(bottom.clone())
                } else if close_window_command_(bottom, window) {
                    Some(top.clone())
                } else {
                    None
                }
            },
        };
        match new_layout {
            Some(new_layout) => {
                *window = find_first_window(&new_layout).clone();
                *layout = *new_layout;
            },
            None => (),
        }
        false
    }
    close_window_command_(&mut state.layout, &mut state.selected_window);
    Ok(())
}

/// Close all other [`Window`]s but the selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
pub fn close_other_windows_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    state.layout = Layout::Window(state.selected_window.clone());
    Ok(())
}

/// Change the selected [`Window`] to the clockwise successor of the
/// current selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
pub fn other_window_clockwise_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    fn other_window_clockwise_command(layout: &Layout, window: &mut Arc<Mutex<Window>>,
                                      top_level: bool) -> bool {
        match layout {
            Layout::Window(w) => Arc::ptr_eq(w, window),
            Layout::VSplit { left, right } => {
                if other_window_clockwise_command(left, window, false) {
                    *window = find_first_window(right).clone();
                    false
                } else if other_window_clockwise_command(right, window, false) {
                    if top_level {
                        *window = find_first_window(left).clone();
                    }
                    true
                } else {
                    false
                }
            },
            Layout::HSplit { top, bottom } => {
                if other_window_clockwise_command(top, window, false) {
                    *window = find_first_window(bottom).clone();
                    false
                } else if other_window_clockwise_command(bottom, window, false) {
                    if top_level {
                        *window = find_first_window(top).clone();
                    }
                    true
                } else {
                    false
                }
            },
        }
    }
    other_window_clockwise_command(&state.layout, &mut state.selected_window, true);
    Ok(())
}

/// Change the selected [`Window`] to the counter clockwise successor
/// of the current selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
pub fn other_window_counter_clockwise_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    fn other_window_counter_clockwise_command(layout: &Layout, window: &mut Arc<Mutex<Window>>,
                                              top_level: bool) -> bool {
        match layout {
            Layout::Window(w) => Arc::ptr_eq(w, window),
            Layout::VSplit { left, right } => {
                if other_window_counter_clockwise_command(right, window, false) {
                    *window = find_last_window(left).clone();
                    false
                } else if other_window_counter_clockwise_command(left, window, false) {
                    if top_level {
                        *window = find_last_window(right).clone();
                    }
                    true
                } else {
                    false
                }
            },
            Layout::HSplit { top, bottom } => {
                if other_window_counter_clockwise_command(bottom, window, false) {
                    *window = find_last_window(top).clone();
                    false
                } else if other_window_counter_clockwise_command(top, window, false) {
                    if top_level {
                        *window = find_last_window(bottom).clone();
                    }
                    true
                } else {
                    false
                }
            },
        }
    }
    other_window_counter_clockwise_command(&state.layout, &mut state.selected_window, true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use split_commands::*;

    #[test]
    fn close_window_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();

        horizontal_split_command(&mut state, &mut display).unwrap();
        {
            let mut selected_window = state.selected_window.lock();
            selected_window.set_cursor(2);
        }
        display.show(&state).unwrap();

        close_window_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!((display.selected_cursors, display.unselected_cursors),
                   (vec![(0, 4)], vec![]));
    }

    #[test]
    fn close_window_command_2() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

        horizontal_split_command(&mut state, &mut display).unwrap();
        vertical_split_command(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.layout.update_window_cursors();
        {
            let mut selected_window = state.selected_window.lock();
            selected_window.set_cursor(2);
        }
        display.show(&state).unwrap();

        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 2)]);
        assert_eq!(display.unselected_cursors, vec![(0, 15), (8, 4)]);

        close_window_command(&mut state, &mut display).unwrap();
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
        assert_eq!((display.selected_cursors, display.unselected_cursors),
                   (vec![(0, 4)], vec![(8, 4)]));
    }

    #[test]
    fn close_other_windows_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

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
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![]);

        {
            let mut selected_window = state.selected_window.lock();
            selected_window.set_cursor(0);
        }
        horizontal_split_command(&mut state, &mut display).unwrap();
        vertical_split_command(&mut state, &mut display).unwrap();
        {
            let mut selected_window = state.selected_window.lock();
            selected_window.set_cursor(2);
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 2)]);
        assert_eq!(display.unselected_cursors, vec![(0, 11), (8, 0)]);

        close_other_windows_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!((display.selected_cursors, display.unselected_cursors),
                   (vec![(0, 2)], vec![]));
    }

    #[test]
    fn other_window_clockwise_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

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
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);

        horizontal_split_command(&mut state, &mut display).unwrap();
        vertical_split_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![(0, 15), (8, 4)]);

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(0, 15)], &vec![(0, 4), (8, 4)]));

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(8, 4)], &vec![(0, 4), (0, 15)]));

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(0, 4)], &vec![(0, 15), (8, 4)]));
    }

    #[test]
    fn other_window_clockwise_command_2() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

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
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>()]);

        vertical_split_command(&mut state, &mut display).unwrap();
        other_window_clockwise_command(&mut state, &mut display).unwrap();
        horizontal_split_command(&mut state, &mut display).unwrap();
        {
            let mut selected_window = state.selected_window.lock();
            selected_window.set_cursor(2);
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 13)]);
        assert_eq!(display.unselected_cursors, vec![(0, 4), (8, 15)]);

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(8, 15)], &vec![(0, 4), (0, 13)]));

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(0, 4)], &vec![(0, 13), (8, 15)]));

        other_window_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!((&display.selected_cursors, &display.unselected_cursors),
                   (&vec![(0, 13)], &vec![(0, 4), (8, 15)]));
    }

    #[test]
    fn other_window_counter_clockwise_command_1() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(Vec::new());

        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }

        vertical_split_command(&mut state, &mut display).unwrap();
        other_window_counter_clockwise_command(&mut state, &mut display).unwrap();
        horizontal_split_command(&mut state, &mut display).unwrap();

        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 15)]);
        assert_eq!(display.unselected_cursors, vec![(0, 4), (8, 15)]);

        other_window_counter_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![(0, 15), (8, 15)]);

        other_window_counter_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(8, 15)]);
        assert_eq!(display.unselected_cursors, vec![(0, 4), (0, 15)]);

        other_window_counter_clockwise_command(&mut state, &mut display).unwrap();
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |---------".chars().collect::<Vec<_>>(),
                        "          |abcd     ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors, vec![(0, 15)]);
        assert_eq!(display.unselected_cursors, vec![(0, 4), (8, 15)]);
    }
}
