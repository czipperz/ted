use git2::*;
use git_common::*;
use git_repository::*;
use parking_lot::Mutex;
use std::path::*;
use std::sync::Arc;
use ted_core::*;

#[derive(Debug)]
pub struct GitAddCommand;

pub fn git_add_command() -> Arc<GitAddCommand> {
    Arc::new(GitAddCommand)
}

impl Command for GitAddCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let (buffer, cursor) = {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            (selected_window.buffer.clone(), selected_window.cursor.get())
        };
        let (path, file) = {
            let buffer = buffer.lock();
            (buffer.name.file_path.as_ref().ok_or_else(|| "Error: Cannot open git repository")?.to_path_buf(),
             get_highlighted_file_unstaged(&buffer, cursor)?)
        };
        git_add(&path, &file)?;
        {
            let mut buffer = buffer.lock();
            git_refresh_repository(&path, &mut buffer)?;
        }
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
        let (buffer, cursor) = {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            (selected_window.buffer.clone(), selected_window.cursor.get())
        };
        let (path, file) = {
            let buffer = buffer.lock();
            (buffer.name.file_path.as_ref().ok_or_else(|| "Error: Cannot open git repository")?.to_path_buf(),
             get_highlighted_file_staged(&buffer, cursor)?)
        };
        git_unstage(&path, &file)?;
        {
            let mut buffer = buffer.lock();
            git_refresh_repository(&path, &mut buffer)?;
        }
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
