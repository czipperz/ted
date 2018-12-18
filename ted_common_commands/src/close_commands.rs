use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::{Command, State};

lazy_static! {
    static ref CLOSED_SUCCESSFULLY: Mutex<bool> = Mutex::new(false);
}

pub fn was_closed_successfully() -> bool {
    *CLOSED_SUCCESSFULLY.lock()
}

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
    fn execute(&self, _: Arc<Mutex<State>>) -> Result<(), String> {
        let mut closed_successfully = CLOSED_SUCCESSFULLY.lock();
        *closed_successfully = true;
        Err("Closed Successfully".to_string())
    }
}
