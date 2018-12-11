use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

pub fn vertical_split(layout: &mut Layout, window: &Arc<Mutex<Window>>) {
    let split = match layout {
        Layout::Window(w) => if Arc::ptr_eq(w, window) {
            Some(w.clone())
        } else {
            None
        },
        Layout::VSplit { left, right } => {
            vertical_split(left, window);
            vertical_split(right, window);
            None
        }
        Layout::HSplit { top, bottom } => {
            vertical_split(top, window);
            vertical_split(bottom, window);
            None
        }
    };
    match split {
        Some(w) => {
            let right = Box::new(Layout::Window(clone_window(window)));
            *layout = Layout::VSplit {
                left: Box::new(Layout::Window(w)),
                right,
            };
        }
        None => {}
    }
}

/// Split the selected [`Window`](../ted_core/struct.Window.html) in
/// two vertically -- that is into a left and right part.
pub fn vertical_split_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_frame = state.lock().display.selected_frame.clone();
    let mut selected_frame = selected_frame.lock();
    let selected_frame = &mut *selected_frame;
    vertical_split(&mut selected_frame.layout, &selected_frame.selected_window);
    Ok(())
}

pub fn horizontal_split(layout: &mut Layout, window: &Arc<Mutex<Window>>) {
    let split = match layout {
        Layout::Window(w) => if Arc::ptr_eq(w, window) {
            Some(w.clone())
        } else {
            None
        },
        Layout::VSplit { left, right } => {
            horizontal_split(left, window);
            horizontal_split(right, window);
            None
        }
        Layout::HSplit { top, bottom } => {
            horizontal_split(top, window);
            horizontal_split(bottom, window);
            None
        }
    };
    match split {
        Some(w) => {
            let bottom = Box::new(Layout::Window(clone_window(window)));;
            *layout = Layout::HSplit {
                top: Box::new(Layout::Window(w)),
                bottom,
            };
        }
        None => {}
    }
}

/// Split the selected [`Window`](../ted_core/struct.Window.html) in
/// two horizontally -- that is into a top and bottom part.
pub fn horizontal_split_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let selected_frame = state.lock().display.selected_frame.clone();
    let mut selected_frame = selected_frame.lock();
    let selected_frame = &mut *selected_frame;
    horizontal_split(&mut selected_frame.layout, &selected_frame.selected_window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_split_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        vertical_split_command(state.clone()).unwrap();
        let selected_frame = state.lock().display.selected_frame.clone();
        let selected_frame = selected_frame.lock();
        match &selected_frame.layout {
            Layout::VSplit { left, right } => {
                let left = match **left {
                    Layout::Window(ref left) => left,
                    _ => unreachable!(),
                };
                let right = match **right {
                    Layout::Window(ref right) => right,
                    _ => unreachable!(),
                };
                assert!(!Arc::ptr_eq(&left, &right));
                let left = left.lock();
                let right = right.lock();
                assert!(Arc::ptr_eq(&left.buffer, &right.buffer));
                assert!(Arc::ptr_eq(&left.buffer_key_map, &right.buffer_key_map));
                assert_eq!(left.cursor, right.cursor);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn horizontal_split_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        horizontal_split_command(state.clone()).unwrap();
        {
            let selected_frame = state.lock().display.selected_frame.clone();
            let selected_frame = selected_frame.lock();
            match &selected_frame.layout {
                Layout::HSplit { top, bottom } => {
                    let top = match **top {
                        Layout::Window(ref top) => top,
                        _ => unreachable!(),
                    };
                    let bottom = match **bottom {
                        Layout::Window(ref bottom) => bottom,
                        _ => unreachable!(),
                    };
                    assert!(!Arc::ptr_eq(&top, &bottom));
                    let top = top.lock();
                    let bottom = bottom.lock();
                    assert!(Arc::ptr_eq(&top.buffer, &bottom.buffer));
                    assert!(Arc::ptr_eq(&top.buffer_key_map, &bottom.buffer_key_map));
                    assert_eq!(top.cursor, bottom.cursor);
                }
                _ => unreachable!(),
            }
        }

        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>(),
                    "--------------------".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }

        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd                ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>(),
                    "--------------------".chars().collect::<Vec<_>>(),
                    "abcd                ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }
    }
}
