use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::State;
pub fn close_ted_command(_: Arc<Mutex<State>>) -> Result<(), ()> {
    Err(())
}
