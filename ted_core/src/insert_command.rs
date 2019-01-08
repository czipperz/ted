use command::Command;
use parking_lot::Mutex;
use state::State;
use std::sync::Arc;

#[derive(Debug)]
struct InsertCommand(char);
pub fn insert_command(key: char) -> Arc<Command> {
    Arc::new(InsertCommand(key))
}

impl Command for InsertCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        selected_window.insert(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use debug_renderer::*;

    #[test]
    fn insert_command_into_empty() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        let window = state.lock().display.selected_window();
        let buffer = window.lock().buffer.clone();

        insert_command(' ').execute(state.clone()).unwrap();
        assert_eq!(buffer.lock().to_string(), " ");
        assert_eq!(window.lock().cursor.get(), 1);
    }

    #[test]
    fn insert_command_at_end() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        let window = state.lock().display.selected_window();
        let buffer = window.lock().buffer.clone();

        window.lock().insert(' ').unwrap();
        assert_eq!(window.lock().cursor.get(), 1);

        insert_command('x').execute(state.clone()).unwrap();
        assert_eq!(buffer.lock().to_string(), " x");
        assert_eq!(window.lock().cursor.get(), 2);
    }

    #[test]
    fn insert_command_at_beginning() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        let window = state.lock().display.selected_window();
        let buffer = window.lock().buffer.clone();

        window.lock().insert(' ').unwrap();
        window.lock().set_cursor(0);

        insert_command('x').execute(state.clone()).unwrap();
        assert_eq!(buffer.lock().to_string(), "x ");
        assert_eq!(window.lock().cursor.get(), 1);
    }
}
