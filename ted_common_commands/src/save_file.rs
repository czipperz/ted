use parking_lot::Mutex;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use ted_core::*;

pub fn save_buffer(buffer: &Buffer) -> Result<(), String> {
    let path = buffer.name.path_result()?;
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    for c in buffer.iter() {
        write!(file, "{}", c).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct SaveFileCommand;

pub fn save_file_command() -> Arc<SaveFileCommand> {
    Arc::new(SaveFileCommand)
}

impl Command for SaveFileCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = state.lock().display.selected_window_buffer();
        let buffer = buffer.lock();
        save_buffer(&*buffer)
    }
}
