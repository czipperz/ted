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

#[cfg(test)]
mod tests {
    use super::*;
    use ted_core::DebugRenderer;

    #[test]
    fn close_ted_command_test() {
        assert!(!was_closed_successfully());
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        let r = CloseTedCommand.execute(state);
        assert!(r.is_err());
        assert!(was_closed_successfully());
    }
}
