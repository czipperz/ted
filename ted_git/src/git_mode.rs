use git_common::*;
use git_diff::*;
use git_repository::*;
use git_stage::*;
use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

lazy_static! {
    static ref GIT_MODE: Arc<Mutex<Mode>> = create_git_mode();
}

fn is_in_git_mode(buffer: &Buffer) -> bool {
    buffer
        .buffer_modes
        .iter()
        .any(|mode| Arc::ptr_eq(mode, &GIT_MODE))
}

fn add_git_mode(buffer_modes: &mut Vec<Arc<Mutex<Mode>>>) {
    buffer_modes.push(git_mode());
}
fn remove_git_mode(buffer_modes: &mut Vec<Arc<Mutex<Mode>>>) {
    buffer_modes.retain(|mode| !Arc::ptr_eq(mode, &GIT_MODE));
}

pub fn check_if_in_git_mode(buffer: &Buffer) -> Result<(), String> {
    if is_in_git_mode(buffer) {
        Ok(())
    } else {
        Err(ERROR_NOT_IN_GIT_MODE)?
    }
}

pub fn turn_on_git_mode(buffer: &mut Buffer) {
    if !is_in_git_mode(buffer) {
        buffer.buffer_modes.push(git_mode());
    }
}

pub fn turn_off_git_mode(buffer: &mut Buffer) {
    if is_in_git_mode(buffer) {
        remove_git_mode(&mut buffer.buffer_modes);
    }
}

pub fn toggle_git_mode(buffer: &mut Buffer) {
    (if is_in_git_mode(buffer) {
        remove_git_mode
    } else {
        add_git_mode
    })(&mut buffer.buffer_modes)
}

fn create_git_mode() -> Arc<Mutex<Mode>> {
    let mode = Mode::new();
    {
        let mut key_map = mode.key_map.lock();
        key_map.bind(vec![kbd("g")], git_refresh_repository_command());
        key_map.bind(vec![kbd("a")], git_add_command());
        key_map.bind(vec![kbd("u")], git_unstage_command());
        key_map.bind(vec![kbd("d")], git_diff_command());
    }
    Arc::new(Mutex::new(mode))
}

pub fn git_mode() -> Arc<Mutex<Mode>> {
    GIT_MODE.clone()
}
