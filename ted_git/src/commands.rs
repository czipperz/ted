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
        let file = get_highlighted_file(&buffer, false, selected_window.cursor.get())?;
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
    let repo = check(Repository::discover(directory))?;
    let mut index = check(repo.index())?;
    check(index.add_path(file))?;
    check(index.write())?;
    Ok(())
}

#[derive(Debug)]
pub struct GitUnstageCommand;

pub fn git_unstage_command() -> Arc<GitUnstageCommand> {
    Arc::new(GitUnstageCommand)
}

impl Command for GitUnstageCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        let mut buffer = selected_window.buffer.lock();
        let file = get_highlighted_file(&buffer, true, selected_window.cursor.get())?;
        git_unstage(buffer.name.file_path.as_ref().unwrap(), &file)?;
        refresh_git_repository(
            &buffer.name.file_path.as_ref().unwrap().clone(),
            &mut buffer,
        )?;
        Ok(())
    }
}

pub fn git_unstage(directory: &Path, file: &Path) -> Result<(), String> {
    log_debug(format!(
        "{}: git unstage {}",
        directory.display().to_string(),
        file.display().to_string()
    ));
    let repo = check(Repository::discover(directory))?;
    let target = check(check(repo.head())?.peel(ObjectType::Commit))?;
    check(repo.reset_default(Some(&target), Some(file)))?;
    Ok(())
}

fn get_highlighted_file(buffer: &Buffer, is_staged: bool, mut cursor: usize) -> Result<PathBuf, String> {
    let initial_cursor = cursor;
    check_if_in_git_mode(&buffer)?;
    cursor = begin_of_line(&buffer, cursor);
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
            if is_staged {
                break;
            } else {
                return Err("Error: File is already staged".to_string());
            }
        }
        if substring == "Unstaged files: " {
            if is_staged {
                return Err("Error: File is not staged".to_string());
            } else {
                break;
            }
        }
        if cursor == 0 {
            return Err(
                "Error: Place the cursor on the line of a file before adding".to_string(),
            );
        }
        cursor = forward_line(&buffer, cursor, -1);
    }
    Ok(PathBuf::from(buffer.substring(
        begin_of_line(&buffer, initial_cursor) + 2,
        end_of_line(&buffer, initial_cursor),
    )?))
}
