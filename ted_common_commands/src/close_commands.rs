use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::{Command, State};

/// Close Ted.
#[derive(Debug)]
pub struct CloseTedCommand;

/// Construct a [`CloseTedCommand`].
///
/// [`CloseTedCommand`]: struct.CloseTedCommand.html
pub fn close_ted_command() -> Arc<CloseTedCommand> {
    Arc::new(CloseTedCommand)
}

impl Command for CloseTedCommand {
    fn execute(&self, _: Arc<Mutex<State>>) -> Result<(), ()> {
        Err(())
    }
}
