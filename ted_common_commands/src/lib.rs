extern crate parking_lot;
extern crate ted_core;

mod delete_commands;
pub use delete_commands::*;

mod move_commands;
pub use move_commands::*;

mod move_group_commands;
pub use move_group_commands::*;

mod split_commands;
pub use split_commands::*;

mod window_commands;
pub use window_commands::*;

mod change_commands;
pub use change_commands::*;

use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::State;
pub fn close_ted_command(_: Arc<Mutex<State>>) -> Result<(), ()> {
    Err(())
}
