use git2::*;
use git_common::*;
use git_mode::*;
use parking_lot::Mutex;
use std::path::*;
use std::sync::Arc;
use ted_core::*;

#[derive(Debug)]
pub struct GitOpenRepositoryCommand;

pub fn git_open_repository_command() -> Arc<GitOpenRepositoryCommand> {
    Arc::new(GitOpenRepositoryCommand)
}

impl Command for GitOpenRepositoryCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let selected_window = selected_frame.lock().selected_window.clone();
        let repository_path = {
            let buffer = selected_window.lock().buffer.clone();
            let buffer = buffer.lock();
            buffer.name.parent().ok_or_else(|| "Error: Cannot open git repository".to_string())?.to_path_buf()
        };
        let window = git_open_repository(&repository_path)?;
        let window = Arc::new(Mutex::new(window));
        let mut selected_frame = selected_frame.lock();
        selected_frame.layout.replace_selected_window(&selected_window, window.clone());
        selected_frame.selected_window = window;
        Ok(())
    }
}

pub fn git_open_repository(repository_path: &Path) -> Result<Window, String> {
    let mut buffer = Buffer::new(repository_path.into());
    git_refresh_repository(&repository_path, &mut buffer)?;
    Ok(Window::from(buffer))
}

#[derive(Debug)]
pub struct GitRefreshRepositoryCommand;

pub fn git_refresh_repository_command() -> Arc<GitRefreshRepositoryCommand> {
    Arc::new(GitRefreshRepositoryCommand)
}

impl Command for GitRefreshRepositoryCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let buffer = state.lock().display.selected_window_buffer();
        let mut buffer = buffer.lock();
        let repository_path = buffer.name.path.clone().ok_or_else(|| ERROR_FILE_PATH_NONE)?;
        git_refresh_repository(&repository_path, &mut buffer)?;
        Ok(())
    }
}

pub fn git_refresh_repository(repository_path: &Path, buffer: &mut Buffer) -> Result<(), String> {
    buffer.read_only = false;
    buffer.buffer_modes.push(git_mode());
    let mut buf = String::new();
    let repo = check(Repository::discover(repository_path))?;
    let workdir = repo.workdir().ok_or_else(|| ERROR_REPOSITORY_WORKDIR_NONE)?;
    buffer.name = BufferName {
        name: format!("*git* {}", workdir.file_name().ok_or_else(|| ERROR_REPOSITORY_WORKDIR_NONE)?
                      .to_str().ok_or_else(|| ERROR_REPOSITORY_WORKDIR_TO_STR_NONE)?),
        path: Some(workdir.to_path_buf()),
    };
    buf.push_str(&workdir.display().to_string());
    buf.push_str(": ");
    buf.push_str(&format!("{:?}", repo.state()));
    buf.push('\n');
    let statuses = check(repo.statuses(None))?;
    if !statuses.is_empty() {
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        for status in statuses.iter() {
            use std::rc::Rc;
            let file = Rc::new(status.path().unwrap().to_string());
            let stat = status.status();
            if stat.is_index_new()
                | stat.is_index_modified()
                | stat.is_index_deleted()
                | stat.is_index_typechange()
                | stat.is_index_renamed()
            {
                staged.push((file.clone(), stat));
            }
            if stat.is_wt_new()
                | stat.is_wt_modified()
                | stat.is_wt_deleted()
                | stat.is_wt_typechange()
                | stat.is_wt_renamed()
            {
                if !stat.is_ignored() {
                    unstaged.push((file, stat));
                }
            }
        }

        if !staged.is_empty() {
            buf.push_str("\nStaged files:\n");
        }
        for (file, stat) in staged {
            if stat.is_index_new() {
                buf.push('N');
            } else if stat.is_index_modified() {
                buf.push('M');
            } else if stat.is_index_deleted() {
                buf.push('D');
            } else if stat.is_index_typechange() {
                buf.push('T');
            } else if stat.is_index_renamed() {
                buf.push('R');
            } else {
                unreachable!();
            }
            buf.push(' ');
            buf.push_str(&file);
            buf.push('\n');
        }

        if !unstaged.is_empty() {
            buf.push_str("\nUnstaged files:\n");
        }
        for (file, stat) in unstaged {
            if stat.is_wt_new() {
                buf.push('N');
            } else if stat.is_wt_modified() {
                buf.push('M');
            } else if stat.is_wt_deleted() {
                buf.push('D');
            } else if stat.is_wt_typechange() {
                buf.push('T');
            } else if stat.is_wt_renamed() {
                buf.push('R');
            } else {
                unreachable!();
            }
            buf.push(' ');
            buf.push_str(&file);
            buf.push('\n');
        }
    }
    buffer.clear()?;
    buffer.insert_str(0, &buf)?;
    buffer.erase_history();
    buffer.read_only = true;
    Ok(())
}
