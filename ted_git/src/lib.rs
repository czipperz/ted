extern crate git2;
extern crate parking_lot;
extern crate ted_core;

use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ted_core::*;

mod refresh_git_repository;
pub use refresh_git_repository::*;

#[derive(Debug)]
pub struct OpenGitRepository;

pub fn open_git_repository_command() -> Arc<OpenGitRepository> {
    Arc::new(OpenGitRepository)
}

impl Command for OpenGitRepository {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let window = open_git_repository(PathBuf::from("/home/czipperz/ted/src"))?;
        let window = Arc::new(Mutex::new(window));
        let selected_frame = state.lock().display.selected_frame.clone();
        selected_frame.lock().replace_selected_window(window);
        Ok(())
    }
}

#[derive(Debug)]
pub struct RefreshGitRepository;

pub fn refresh_git_repository_command() -> Arc<RefreshGitRepository> {
    Arc::new(RefreshGitRepository)
}

impl Command for RefreshGitRepository {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = state.lock().display.selected_window_buffer();
        refresh_git_repository(Path::new("/home/czipperz/ted/src"), &mut *buffer.lock())?;
        Ok(())
    }
}

pub fn open_git_repository(path: PathBuf) -> Result<Window, String> {
    let mut buffer = Buffer::new(path.clone().into());
    refresh_git_repository(&path, &mut buffer)?;
    Ok(Window::from(buffer))
}
