use git2::*;
use git_mode::check_if_in_git_mode;
use parking_lot::Mutex;
use refresh_git_repository::*;
use std::path::*;
use std::sync::Arc;
use ted_common_commands::*;
use ted_core::*;

#[derive(Debug)]
pub struct OpenGitRepository;

pub fn open_git_repository_command() -> Arc<OpenGitRepository> {
    Arc::new(OpenGitRepository)
}

impl Command for OpenGitRepository {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let path = {
            let selected_window_buffer = state.lock().display.selected_window_buffer();
            let selected_window_buffer = selected_window_buffer.lock();
            match selected_window_buffer
                .name
                .file_path
                .as_ref()
                .and_then(|p| p.parent())
            {
                Some(file_path) => file_path.to_path_buf(),
                None => return Err("Error: Cannot open git repository".to_string()),
            }
        };
        let window = open_git_repository(&path)?;
        let window = Arc::new(Mutex::new(window));
        let selected_frame = state.lock().display.selected_frame.clone();
        selected_frame.lock().replace_selected_window(window);
        Ok(())
    }
}

pub fn open_git_repository(path: &Path) -> Result<Window, String> {
    let mut buffer = Buffer::new(path.into());
    refresh_git_repository(&path, &mut buffer)?;
    Ok(Window::from(buffer))
}

#[derive(Debug)]
pub struct RefreshGitRepository;

pub fn refresh_git_repository_command() -> Arc<RefreshGitRepository> {
    Arc::new(RefreshGitRepository)
}

impl Command for RefreshGitRepository {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = state.lock().display.selected_window_buffer();
        let mut buffer = buffer.lock();
        match buffer.name.file_path.clone() {
            Some(path) => refresh_git_repository(&path, &mut buffer),
            None => Err("Error: Not a git repository".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct GitAddCommand;

pub fn git_add_command() -> Arc<GitAddCommand> {
    Arc::new(GitAddCommand)
}

impl Command for GitAddCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        let mut buffer = selected_window.buffer.lock();
        check_if_in_git_mode(&buffer)?;
        let mut cursor = begin_of_line(&buffer, selected_window.cursor.get());
        {
            let prev_line = forward_line(&buffer, cursor, -1);
            if prev_line == cursor {
                return Err(
                    "Error: Place the cursor on the line of a file before adding".to_string(),
                );
            }
            cursor = prev_line;
        }
        loop {
            let substring = buffer.substring(cursor, end_of_line(&buffer, cursor))?;
            if substring == "Staged files: " {
                return Err("Error: File is already added".to_string());
            }
            if substring == "Unstaged files: " {
                break;
            }
            if cursor == 0 {
                return Err(
                    "Error: Place the cursor on the line of a file before adding".to_string(),
                );
            }
            cursor = forward_line(&buffer, cursor, -1);
        }
        let file = PathBuf::from(buffer.substring(
            begin_of_line(&buffer, selected_window.cursor.get()) + 2,
            end_of_line(&buffer, selected_window.cursor.get()),
        )?);
        git_add(buffer.name.file_path.as_ref().unwrap(), &file)?;
        refresh_git_repository(
            &buffer.name.file_path.as_ref().unwrap().clone(),
            &mut buffer,
        )?;
        Ok(())
    }
}

pub fn git_add(directory: &Path, file: &Path) -> Result<(), String> {
    log_debug(format!(
        "{}: git add {}",
        directory.display().to_string(),
        file.display().to_string()
    ));
    let repo = Repository::discover(directory).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index.add_path(file).map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;
    Ok(())
}
