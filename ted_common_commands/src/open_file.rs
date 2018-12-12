use parking_lot::Mutex;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Arc;
use ted_core::*;

pub fn open_file(name: String, path: String) -> Result<Buffer, ()> {
    let file = File::open(&path).map_err(|_| ())?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf).map_err(|_| ())?;
    Ok(Buffer::new_with_contents(
        BufferName::File { name, path },
        &buf,
    ))
}

fn display_window(selected_frame: Arc<Mutex<Frame>>, window: Arc<Mutex<Window>>) {
    let mut selected_frame = selected_frame.lock();
    let selected_frame = &mut *selected_frame;
    selected_frame
        .layout
        .set_selected_window(&selected_frame.selected_window, Layout::Window(window));
}

pub fn open_file_command(state: Arc<Mutex<State>>) -> Result<(), ()> {
    let buffer = open_file(
        "main.rs".to_string(),
        "/home/czipperz/ted/src/main.rs".to_string(),
    )?;
    let window = Arc::new(Mutex::new(Window::from(buffer)));
    let selected_frame = state.lock().display.selected_frame.clone();
    display_window(selected_frame, window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_file() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        open_file_command(state.clone()).unwrap();
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
