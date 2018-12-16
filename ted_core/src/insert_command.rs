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
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        selected_window.insert(self.0)
    }
}
