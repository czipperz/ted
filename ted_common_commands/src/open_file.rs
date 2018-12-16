use parking_lot::Mutex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;
use ted_core::*;

pub fn open_file(path: PathBuf) -> Result<Buffer, ()> {
    let path = path.canonicalize().map_err(|_| ())?;
    if !path.exists() {
        Err(())
    } else if path.is_dir() {
        let mut buf = String::new();
        buf.push_str(&path.display().to_string());
        buf.push('\n');
        for entry in path.read_dir().map_err(|_| ())? {
            if let Ok(entry) = entry {
                buf.push_str(entry.path().to_str().unwrap());
                buf.push('\n');
            }
        }
        Ok(Buffer::new_with_contents(path.into(), &buf))
    } else {
        let file = File::open(&path).map_err(|_| ())?;
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).map_err(|_| ())?;
        Ok(Buffer::new_with_contents(path.into(), &buf))
    }
}

#[derive(Debug)]
pub struct OpenFileCommand;

/// Construct a [`OpenFileCommand`].
///
/// [`OpenFileCommand`]: struct.OpenFileCommand.html
pub fn open_file_command() -> Arc<OpenFileCommand> {
    Arc::new(OpenFileCommand)
}

impl Command for OpenFileCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let buffer = open_file("/home/czipperz/ted/src/main.rs".into())?;
        let window = Arc::new(Mutex::new(Window::from(buffer)));
        let selected_frame = state.lock().display.selected_frame.clone();
        selected_frame.lock().replace_selected_window(window);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_file() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        open_file_command().execute(state.clone()).unwrap();
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        assert!(
            selected_window
                .cursor
                .is_updated(&selected_window.buffer.lock())
        );
        assert_eq!(selected_window.cursor.get(), 0);
    }
}
