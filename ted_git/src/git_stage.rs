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
        let (repository_path, file) = {
            let buffer = buffer.lock();
            (buffer.name.path.as_ref().ok_or(ERROR_REPOSITORY_WORKDIR_NONE)?.to_path_buf(),
             get_highlighted_file_unstaged(&buffer, cursor)?)
        };
        git_add(&repository_path, &file)?;
        {
            let mut buffer = buffer.lock();
            git_refresh_repository(&repository_path, &mut buffer)?;
        }
        Ok(())
    }
}

pub fn git_add(repository_path: &Path, file: &Path) -> Result<(), String> {
    log_debug(format!(
        "{}: git add {}",
        repository_path.display().to_string(),
        file.display().to_string()
    ));
    let repository = check(Repository::discover(repository_path))?;
    let mut index = check(repository.index())?;
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
        let (repository_path, file) = {
            let buffer = buffer.lock();
            (buffer.name.path.as_ref().ok_or(ERROR_REPOSITORY_WORKDIR_NONE)?.to_path_buf(),
             get_highlighted_file_staged(&buffer, cursor)?)
        };
        git_unstage(&repository_path, &file)?;
        {
            let mut buffer = buffer.lock();
            git_refresh_repository(&repository_path, &mut buffer)?;
        }
        Ok(())
    }
}

pub fn git_unstage(repository_path: &Path, file: &Path) -> Result<(), String> {
    log_debug(format!(
        "{}: git unstage {}",
        repository_path.display().to_string(),
        file.display().to_string()
    ));
    let repository = check(Repository::discover(repository_path))?;
    let target = check(check(repository.head())?.peel(ObjectType::Commit))?;
    check(repository.reset_default(Some(&target), Some(file)))?;
    Ok(())
}
