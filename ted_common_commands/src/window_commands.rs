use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

pub fn close_window(layout: &mut Layout, window: &mut Arc<Mutex<Window>>) -> bool {
    let new_layout = match layout {
        Layout::Window(w) => return Arc::ptr_eq(w, window),
        Layout::VSplit { left, right } => {
            if close_window(left, window) {
                Some(right.clone())
            } else if close_window(right, window) {
                Some(left.clone())
            } else {
                None
            }
        }
        Layout::HSplit { top, bottom } => {
            if close_window(top, window) {
                Some(bottom.clone())
            } else if close_window(bottom, window) {
                Some(top.clone())
            } else {
                None
            }
        }
    };
    match new_layout {
        Some(new_layout) => {
            *window = new_layout.first_window().clone();
            *layout = *new_layout;
        }
        None => (),
    }
    false
}

/// Close the selected [`Window`] and select the next one in a clockwise fashion.
///
/// If the [`Layout`] is only a single [`Window`], then nothing happens.
/// Otherwise, the immediate parent split is simplified to the other [`Layout`].
///
/// [`Window`]:../ted_core/struct.Window.html
/// [`Layout`]:../ted_core/enum.Layout.html
#[derive(Debug)]
pub struct CloseWindowCommand;

/// Construct a [`CloseWindowCommand`].
///
/// [`CloseWindowCommand`]: struct.CloseWindowCommand.html
pub fn close_window_command() -> Arc<CloseWindowCommand> {
    Arc::new(CloseWindowCommand)
}

impl Command for CloseWindowCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        close_window(
            &mut selected_frame.layout,
            &mut selected_frame.selected_window,
        );
        Ok(())
    }
}

/// Close all other [`Window`]s but the selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
#[derive(Debug)]
pub struct CloseOtherWindowsCommand;

/// Construct a [`CloseOtherWindowsCommand`].
///
/// [`CloseOtherWindowsCommand`]: struct.CloseOtherWindowsCommand.html
pub fn close_other_windows_command() -> Arc<CloseOtherWindowsCommand> {
    Arc::new(CloseOtherWindowsCommand)
}

impl Command for CloseOtherWindowsCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        selected_frame.layout = Layout::Window(selected_frame.selected_window.clone());
        Ok(())
    }
}

pub fn other_window_clockwise(layout: &Layout, window: &mut Arc<Mutex<Window>>) {
    fn other_window_clockwise(
        layout: &Layout,
        window: &mut Arc<Mutex<Window>>,
        top_level: bool,
    ) -> bool {
        match layout {
            Layout::Window(w) => Arc::ptr_eq(w, window),
            Layout::VSplit { left, right } => {
                if other_window_clockwise(left, window, false) {
                    *window = right.first_window().clone();
                    false
                } else if other_window_clockwise(right, window, false) {
                    if top_level {
                        *window = left.first_window().clone();
                    }
                    true
                } else {
                    false
                }
            }
            Layout::HSplit { top, bottom } => {
                if other_window_clockwise(top, window, false) {
                    *window = bottom.first_window().clone();
                    false
                } else if other_window_clockwise(bottom, window, false) {
                    if top_level {
                        *window = top.first_window().clone();
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
    other_window_clockwise(layout, window, true);
}

/// Change the selected [`Window`] to the clockwise successor of the
/// current selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
#[derive(Debug)]
pub struct OtherWindowClockwiseCommand;

/// Construct a [`OtherWindowClockwiseCommand`].
///
/// [`OtherWindowClockwiseCommand`]: struct.OtherWindowClockwiseCommand.html
pub fn other_window_clockwise_command() -> Arc<OtherWindowClockwiseCommand> {
    Arc::new(OtherWindowClockwiseCommand)
}

impl Command for OtherWindowClockwiseCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        other_window_clockwise(&selected_frame.layout, &mut selected_frame.selected_window);
        Ok(())
    }
}

pub fn other_window_counter_clockwise(layout: &Layout, window: &mut Arc<Mutex<Window>>) {
    fn other_window_counter_clockwise(
        layout: &Layout,
        window: &mut Arc<Mutex<Window>>,
        top_level: bool,
    ) -> bool {
        match layout {
            Layout::Window(w) => Arc::ptr_eq(w, window),
            Layout::VSplit { left, right } => {
                if other_window_counter_clockwise(right, window, false) {
                    *window = left.last_window().clone();
                    false
                } else if other_window_counter_clockwise(left, window, false) {
                    if top_level {
                        *window = right.last_window().clone();
                    }
                    true
                } else {
                    false
                }
            }
            Layout::HSplit { top, bottom } => {
                if other_window_counter_clockwise(bottom, window, false) {
                    *window = top.last_window().clone();
                    false
                } else if other_window_counter_clockwise(top, window, false) {
                    if top_level {
                        *window = bottom.last_window().clone();
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
    other_window_counter_clockwise(layout, window, true);
}

/// Change the selected [`Window`] to the counter clockwise successor
/// of the current selected [`Window`].
///
/// [`Window`]: ../ted_core/struct.Window.html
#[derive(Debug)]
pub struct OtherWindowCounterClockwiseCommand;

/// Construct a [`OtherWindowCounterClockwiseCommand`].
///
/// [`OtherWindowCounterClockwiseCommand`]: struct.OtherWindowCounterClockwiseCommand.html
pub fn other_window_counter_clockwise_command() -> Arc<OtherWindowCounterClockwiseCommand> {
    Arc::new(OtherWindowCounterClockwiseCommand)
}

impl Command for OtherWindowCounterClockwiseCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_frame = selected_frame.lock();
        let selected_frame = &mut *selected_frame;
        other_window_counter_clockwise(&selected_frame.layout, &mut selected_frame.selected_window);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use split_commands::*;

    #[test]
    fn close_window_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.lock().display.show().unwrap();

        horizontal_split_command().execute(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            selected_window.set_cursor(2);
        }
        state.lock().display.show().unwrap();

        close_window_command().execute(state.clone()).unwrap();
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
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![]
            );
        }
    }

    #[test]
    fn close_window_command_2() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        horizontal_split_command().execute(state.clone()).unwrap();
        vertical_split_command().execute(state.clone()).unwrap();
        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.lock().display.update_cursors();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            selected_window.set_cursor(2);
        }
        state.lock().display.show().unwrap();

        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 2)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 15), (8, 4)]
            );
        }

        close_window_command().execute(state.clone()).unwrap();
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(8, 4)]
            );
        }
    }

    #[test]
    fn close_other_windows_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
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
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![]
            );
        }

        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            selected_window.set_cursor(0);
        }
        horizontal_split_command().execute(state.clone()).unwrap();
        vertical_split_command().execute(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            selected_window.set_cursor(2);
        }
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 2)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 11), (8, 0)]
            );
        }

        close_other_windows_command()
            .execute(state.clone())
            .unwrap();
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
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 2)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![]
            );
        }
    }

    #[test]
    fn other_window_clockwise_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
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
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
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

        horizontal_split_command().execute(state.clone()).unwrap();
        vertical_split_command().execute(state.clone()).unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 15), (8, 4)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 15)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (8, 4)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(8, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (0, 15)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>(),
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
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 15), (8, 4)]
            );
        }
    }

    #[test]
    fn other_window_clockwise_command_2() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
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
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
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

        vertical_split_command().execute(state.clone()).unwrap();
        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        horizontal_split_command().execute(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let mut selected_window = selected_window.lock();
            selected_window.set_cursor(2);
        }
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 13)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (8, 15)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(8, 15)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (0, 13)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 13), (8, 15)]
            );
        }

        other_window_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 13)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (8, 15)]
            );
        }
    }

    #[test]
    fn other_window_counter_clockwise_command_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));

        {
            let buffer = state.lock().display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }

        vertical_split_command().execute(state.clone()).unwrap();
        other_window_counter_clockwise_command()
            .execute(state.clone())
            .unwrap();
        horizontal_split_command().execute(state.clone()).unwrap();

        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 15)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (8, 15)]
            );
        }

        other_window_counter_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 4)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 15), (8, 15)]
            );
        }

        other_window_counter_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(8, 15)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (0, 15)]
            );
        }

        other_window_counter_clockwise_command()
            .execute(state.clone())
            .unwrap();
        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcd      |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |*scratch*".chars().collect::<Vec<_>>(),
                    "          |---------".chars().collect::<Vec<_>>(),
                    "          |abcd     ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "          |         ".chars().collect::<Vec<_>>(),
                    "*scratch* |*scratch*".chars().collect::<Vec<_>>()
                ]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.selected_cursors,
                vec![(0, 15)]
            );
            assert_eq!(
                unsafe { state.display.debug_renderer() }.unselected_cursors,
                vec![(0, 4), (8, 15)]
            );
        }
    }
}
