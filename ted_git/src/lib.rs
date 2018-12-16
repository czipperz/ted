extern crate git2;
extern crate parking_lot;
extern crate ted_core;

use git2::*;
use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

pub fn open_git_repository(state: Arc<Mutex<State>>) -> Result<(), ()> {
    Ok(())
}
