extern crate git2;
extern crate parking_lot;
extern crate ted_core;

use git2::*;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ted_core::*;

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

pub fn refresh_git_repository(path: &Path, buffer: &mut Buffer) -> Result<(), String> {
    buffer.read_only = false;
    let mut buf = String::new();
    let repo = Repository::discover(path).map_err(|e| e.to_string())?;
    buf.push_str(&repo.workdir().unwrap().display().to_string());
    buf.push_str(": ");
    buf.push_str(&format!("{:?}", repo.state()));
    buf.push('\n');
    let statuses = repo.statuses(None).map_err(|e| e.to_string())?;
    if !statuses.is_empty() {
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        for status in statuses.iter() {
            use std::rc::Rc;
            let file = Rc::new(
                String::from_utf8_lossy(status.path_bytes())
                    .to_owned()
                    .to_string(),
            );
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
            buf.push_str("\nStaged files: \n");
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
            buf.push_str("\nUnstaged files: \n");
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
