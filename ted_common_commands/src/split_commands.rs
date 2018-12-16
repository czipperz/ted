use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

/// Split the selected [`Window`](../ted_core/struct.Window.html) in
/// two vertically -- that is into a left and right part.
#[derive(Debug)]
pub struct VerticalSplitCommand;

/// Construct a [`VerticalSplitCommand`].
///
/// [`VerticalSplitCommand`]: struct.VerticalSplitCommand.html
pub fn vertical_split_command() -> Arc<VerticalSplitCommand> {
    Arc::new(VerticalSplitCommand)
}

impl Command for VerticalSplitCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        let window = &selected_frame.selected_window;
        selected_frame.layout.replace_selected_window(
            &window,
            Layout::VSplit {
                left: Box::new(Layout::Window(window.clone())),
                right: Box::new(Layout::Window(clone_window(window))),
            },
        );
        Ok(())
    }
}

/// Split the selected [`Window`](../ted_core/struct.Window.html) in
/// two horizontally -- that is into a top and bottom part.
#[derive(Debug)]
pub struct HorizontalSplitCommand;

/// Construct a [`HorizontalSplitCommand`].
///
/// [`HorizontalSplitCommand`]: struct.HorizontalSplitCommand.html
pub fn horizontal_split_command() -> Arc<HorizontalSplitCommand> {
    Arc::new(HorizontalSplitCommand)
}

impl Command for HorizontalSplitCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        let window = &selected_frame.selected_window;
        selected_frame.layout.replace_selected_window(
            &window,
            Layout::HSplit {
                top: Box::new(Layout::Window(window.clone())),
                bottom: Box::new(Layout::Window(clone_window(window))),
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_split_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        vertical_split_command().execute(state.clone()).unwrap();
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
        horizontal_split_command().execute(state.clone()).unwrap();
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
