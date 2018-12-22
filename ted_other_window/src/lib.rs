extern crate parking_lot;
extern crate ted_common_commands;
extern crate ted_core;

use parking_lot::Mutex;
use std::sync::Arc;
use ted_common_commands::*;
use ted_core::*;

#[derive(Debug)]
pub struct WithOtherWindowClockwiseCommand<T> {
    command: T,
}

pub fn with_other_window_clockwise<T>(command: T) -> Arc<WithOtherWindowClockwiseCommand<T>>
where
    T: Command,
{
    Arc::new(WithOtherWindowClockwiseCommand { command })
}

impl<T> Command for WithOtherWindowClockwiseCommand<T>
where
    T: Command,
{
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let mut selected_window = selected_frame.lock().selected_window.clone();
        other_window_clockwise(&selected_frame.lock().layout, &mut selected_window);
        std::mem::swap(
            &mut selected_frame.lock().selected_window,
            &mut selected_window,
        );
        let r = self.command.execute(state);
        std::mem::swap(
            &mut selected_frame.lock().selected_window,
            &mut selected_window,
        );
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub struct AssertAtWindowCommand {
        window: Arc<Mutex<Window>>,
    }

    use std::fmt;
    impl fmt::Debug for AssertAtWindowCommand {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "AssertAtWindowCommand")
        }
    }

    pub fn assert_at_window_command(window: Arc<Mutex<Window>>) -> Arc<AssertAtWindowCommand> {
        Arc::new(AssertAtWindowCommand { window })
    }

    impl Command for AssertAtWindowCommand {
        fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
            if Arc::ptr_eq(&self.window, &state.lock().display.selected_window()) {
                Ok(())
            } else {
                Err("Command executed on the wrong window".to_string())
            }
        }
    }

    #[test]
    fn with_other_window_clockwise_no_other_window() {
        let state = State::new(DebugRenderer::new());
        let window = state.display.selected_window();
        let state = Arc::new(Mutex::new(state));

        let at_this = assert_at_window_command(window);
        at_this.execute(state.clone()).unwrap();
        with_other_window_clockwise(at_this)
            .execute(state.clone())
            .unwrap();
    }

    #[test]
    fn with_other_window_clockwise_two_windows() {
        let state = State::new(DebugRenderer::new());
        let window = state.display.selected_window();
        let other = Arc::new(Mutex::new(Window::new()));
        state.display.selected_frame.lock().layout = Layout::VSplit {
            left: window.clone().into(),
            right: other.clone().into(),
        };
        let state = Arc::new(Mutex::new(state));

        let at_this = assert_at_window_command(window);
        assert!(at_this.execute(state.clone()).is_ok());
        assert!(
            with_other_window_clockwise(at_this)
                .execute(state.clone())
                .is_err()
        );

        let at_other = assert_at_window_command(other);
        assert!(at_other.execute(state.clone()).is_err());
        assert!(
            with_other_window_clockwise(at_other)
                .execute(state.clone())
                .is_ok()
        );
    }

    #[test]
    fn with_other_window_clockwise_three_windows() {
        let state = State::new(DebugRenderer::new());
        let this = state.display.selected_window();
        let other = Arc::new(Mutex::new(Window::new()));
        state.display.selected_frame.lock().layout = Layout::VSplit {
            left: this.clone().into(),
            right: Layout::HSplit {
                top: other.clone().into(),
                bottom: Layout::default().into(),
            }.into(),
        };
        let state = Arc::new(Mutex::new(state));

        let at_this = assert_at_window_command(this);
        assert!(at_this.execute(state.clone()).is_ok());
        assert!(
            with_other_window_clockwise(at_this)
                .execute(state.clone())
                .is_err()
        );

        let at_other = assert_at_window_command(other);
        assert!(at_other.execute(state.clone()).is_err());
        assert!(
            with_other_window_clockwise(at_other)
                .execute(state.clone())
                .is_ok()
        );
    }
}
