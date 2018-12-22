use git2::*;
use git_common::*;
use git_repository::*;
use parking_lot::Mutex;
use std::path::Path;
use std::str;
use std::sync::Arc;
use ted_core::*;

#[derive(Debug)]
pub struct GitDiffCommand;

pub fn git_diff_command() -> Arc<GitDiffCommand> {
    Arc::new(GitDiffCommand)
}

impl Command for GitDiffCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_frame = state.lock().display.selected_frame.clone();
        let selected_window = selected_frame.lock().selected_window.clone();
        let (buffer, cursor) = {
            let selected_window = selected_window.lock();
            (selected_window.buffer.clone(),
             selected_window.cursor.get())
        };
        let (repository_path, (file, is_staged)) = {
            let buffer = buffer.lock();
            (buffer.name.path.as_ref().ok_or_else(|| ERROR_FILE_PATH_NONE)?.clone(),
             get_highlighted_file(&buffer, cursor)?)
        };
        {
            let diff_text = git_diff(&repository_path, &file, is_staged)?;
            let buffer = {
                let abs_path = repository_path.join(file);
                let mut buffer = Buffer::new_with_contents(
                    BufferName {
                        name: format!("*git diff* {}", abs_path.display()),
                        path: Some(abs_path),
                    },
                    &diff_text);
                buffer.read_only = true;
                buffer
            };
            let window = Arc::new(Mutex::new(Window::from(buffer)));
            let mut selected_frame = selected_frame.lock();
            selected_frame.layout.replace_selected_window(&selected_window, window.clone());
            selected_frame.selected_window = window;
        }
        {
            let mut buffer = buffer.lock();
            git_refresh_repository(&repository_path, &mut buffer)?;
        }
        Ok(())
    }
}

pub fn git_diff(repository_path: &Path, file: &Path, is_staged: bool) -> Result<String, String> {
    log_debug(format!(
        "{}: git diff {}",
        repository_path.display().to_string(),
        file.display().to_string()
    ));
    let repo = check(Repository::discover(repository_path))?;
    let mut options = DiffOptions::new();
    options.pathspec(file);
    let diff = check(if is_staged {
        let head = check(repo.revparse_single("HEAD^{tree}"))?;
        let head = check(repo.find_tree(head.id()))?;
        repo.diff_tree_to_index(Some(&head), None, Some(&mut options))
    } else {
        repo.diff_index_to_workdir(None, Some(&mut options))
    })?;
    let diff_text = Mutex::new(String::new());
    check(diff.foreach(
        &mut |delta, _progress| {
            let old_file = delta.old_file().path().unwrap();
            let new_file = delta.new_file().path().unwrap();
            let str = if old_file != new_file {
                format!("{} vs {}\n", old_file.display(), new_file.display())
            } else {
                format!("{}\n", old_file.display())
            };
            diff_text.lock().push_str(&str);
            true
        },
        Some(&mut |_delta, _binary| {
            diff_text.lock().push_str(&format!("Binary file changed"));
            true
        }),
        Some(&mut |_delta, hunk| {
            diff_text.lock().push_str(&format!("{}", str::from_utf8(hunk.header()).unwrap()));
            true
        }),
        Some(&mut |_delta, _hunk, line| {
            diff_text.lock().push_str(&format!("{}{}", line.origin(), str::from_utf8(line.content()).unwrap()));
            true
        }),
    ))?;
    Ok(diff_text.into_inner())
}
