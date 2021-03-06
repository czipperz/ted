use git_mode::check_if_in_git_mode;
use std::path::*;
use ted_common_commands::*;
use ted_core::*;

pub fn check<O, E: ToString>(r: Result<O, E>) -> Result<O, String> {
    r.map_err(|e| e.to_string())
}

pub fn get_highlighted_file(buffer: &Buffer, mut cursor: usize) -> Result<(PathBuf, bool), String> {
    let initial_cursor = cursor;
    check_if_in_git_mode(&buffer)?;
    cursor = begin_of_line(&buffer, cursor);
    {
        let prev_line = forward_line(&buffer, cursor, -1);
        if prev_line == cursor {
            Err(ERROR_NO_FILE_HIGHLIGHTED)?;
        }
        cursor = prev_line;
    }
    let is_staged = loop {
        let substring = buffer.substring(cursor, end_of_line(&buffer, cursor))?;
        if substring == "Staged files:" {
            break true;
        }
        if substring == "Unstaged files:" {
            break false;
        }
        if cursor == 0 {
            Err(ERROR_NO_FILE_HIGHLIGHTED)?;
        }
        cursor = forward_line(&buffer, cursor, -1);
    };
    let path = PathBuf::from(buffer.substring(
        begin_of_line(&buffer, initial_cursor) + 2,
        end_of_line(&buffer, initial_cursor),
    )?);
    Ok((path, is_staged))
}

pub fn get_highlighted_file_staged(buffer: &Buffer, cursor: usize) -> Result<PathBuf, String> {
    let (path, is_staged) = get_highlighted_file(buffer, cursor)?;
    if !is_staged {
        Err(ERROR_UNSTAGED)?
    }
    Ok(path)
}

pub fn get_highlighted_file_unstaged(buffer: &Buffer, cursor: usize) -> Result<PathBuf, String> {
    let (path, is_staged) = get_highlighted_file(buffer, cursor)?;
    if is_staged {
        Err(ERROR_STAGED)?
    }
    Ok(path)
}

pub const ERROR_FILE_PATH_NONE: &'static str = "Error: Not a git repository";
pub const ERROR_REPOSITORY_WORKDIR_NONE: &'static str =
    "Error: Git repository has no associated directory";
pub const ERROR_REPOSITORY_WORKDIR_TO_STR_NONE: &'static str =
    "Error: Git repository directory is not valid unicode";
pub const ERROR_NOT_IN_GIT_MODE: &'static str = "Error: Not in git mode";
pub const ERROR_NO_FILE_HIGHLIGHTED: &'static str =
    "Error: Place the cursor on the line of a file before adding";
pub const ERROR_STAGED: &'static str = "Error: File is staged";
pub const ERROR_UNSTAGED: &'static str = "Error: File is unstaged";
