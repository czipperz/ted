use command::*;
use display::Display;
use input::*;
use insert_command::insert_command;
use key_map::*;
use logger::log;
use mode::*;
use parking_lot::Mutex;
use renderer::Renderer;
use std::collections::VecDeque;
use std::sync::Arc;
use window::Window;

/// The state of the program is stored here.
pub struct State {
    pub default_key_map: Arc<Mutex<KeyMap>>,
    pub global_modes: Vec<Arc<Mutex<Mode>>>,
    pub display: Display,
}

impl State {
    /// Create a new `State` with an empty [`Window`].
    ///
    /// [`Window`]: struct.Window.html
    pub fn new<R: 'static + Renderer>(renderer: R) -> Self {
        State {
            default_key_map: Arc::new(Mutex::new(KeyMap::default())),
            global_modes: Vec::new(),
            display: Display::new(Arc::new(Mutex::new(Window::new())), Box::new(renderer)),
        }
    }

    /// This function looks up what [`Command`] an input is bound to.
    ///
    /// First this looks up key bindings on each [`Mode`].  If one is
    /// found, it immediately returns it.  If a mapping is not found
    /// in a [`Mode`], the [`FallthroughBehavior`] is checked.
    ///
    /// For more information on how this function works, look at
    /// [`KeyMap::lookup`].
    ///
    /// [`Command`]: type.Command.html
    /// [`KeyMap::lookup`]: struct.KeyMap.html#method.lookup
    /// [`Mode`]: struct.Mode.html
    pub fn lookup(&self, inputs: &mut VecDeque<Input>) -> Result<Arc<Command>, Result<(), ()>> {
        {
            let buffer = self.display.selected_window_buffer();
            let buffer = buffer.lock();
            for mode in &buffer.buffer_modes {
                let mode = mode.lock();
                match KeyMap::lookup(&mode.key_map, inputs, true) {
                    Ok(command) => return Ok(command),
                    Err(LookupError::NotEnoughInput) => return Err(Ok(())),
                    Err(LookupError::UnboundInput(_)) => (),
                    Err(LookupError::InputWasMapped) => return self.lookup(inputs),
                }
            }
        }
        {
            let selected_window = self.display.selected_window();
            let selected_window = selected_window.lock();
            for mode in &selected_window.window_modes {
                let mode = mode.lock();
                match KeyMap::lookup(&mode.key_map, inputs, true) {
                    Ok(command) => return Ok(command),
                    Err(LookupError::NotEnoughInput) => return Err(Ok(())),
                    Err(LookupError::UnboundInput(_)) => (),
                    Err(LookupError::InputWasMapped) => return self.lookup(inputs),
                }
            }
        }
        for mode in &self.global_modes {
            let mode = mode.lock();
            match KeyMap::lookup(&mode.key_map, inputs, true) {
                Ok(command) => return Ok(command),
                Err(LookupError::NotEnoughInput) => return Err(Ok(())),
                Err(LookupError::UnboundInput(_)) => (),
                Err(LookupError::InputWasMapped) => return self.lookup(inputs),
            }
        }
        match KeyMap::lookup(&self.default_key_map, inputs, false) {
            Ok(command) => Ok(command),
            Err(LookupError::NotEnoughInput) => Err(Ok(())),
            Err(LookupError::UnboundInput(Some(i))) => insert_key_behavior(i),
            Err(LookupError::UnboundInput(_)) => Err(Err(())),
            Err(LookupError::InputWasMapped) => self.lookup(inputs),
        }
    }
}

fn insert_key_behavior(input: Input) -> Result<Arc<Command>, Result<(), ()>> {
    if input.is_unmodified() {
        match input.key {
            Key::Key(c) if is_displayable(c) => {
                return Ok(insert_command(c));
            }
            _ => (),
        }
    }
    log(format!("Invalid input {:?}", input));
    Err(Err(()))
}

fn is_displayable(c: char) -> bool {
    c == '\n' || c == '\t' || !c.is_control()
}

#[cfg(test)]
mod tests {
    use super::*;
    use debug_renderer::DebugRenderer;

    #[test]
    fn lookup_basic() {
        let state = State::new(DebugRenderer::new());
        let command = blank_command();
        state
            .default_key_map
            .lock()
            .bind(vec![kbd("a")], command.clone());
        assert!(state.lookup(&mut vec![].into()).is_err());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a")].into()).unwrap(),
            &command
        ));
    }

    #[test]
    fn lookup_two_levels() {
        let state = State::new(DebugRenderer::new());
        let command = blank_command();
        state
            .default_key_map
            .lock()
            .bind(vec![kbd("a"), kbd("b")], command.clone());
        assert!(state.lookup(&mut vec![].into()).is_err());
        assert!(state.lookup(&mut vec![kbd("a")].into()).is_err());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a"), kbd("b")].into()).unwrap(),
            &command
        ));
    }

    #[test]
    fn lookup_two_same_key() {
        let state = State::new(DebugRenderer::new());
        let command = blank_command();
        state
            .default_key_map
            .lock()
            .bind(vec![kbd("a"), kbd("a")], command.clone());
        assert!(state.lookup(&mut vec![].into()).is_err());
        assert!(state.lookup(&mut vec![kbd("a")].into()).is_err());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a"), kbd("a")].into()).unwrap(),
            &command
        ));
    }

    #[test]
    fn lookup_with_mapping() {
        let state = State::new(DebugRenderer::new());
        let command = blank_command();
        state
            .default_key_map
            .lock()
            .bind(vec![kbd("a"), kbd("b")], command.clone());
        state
            .default_key_map
            .lock()
            .map(vec![kbd("b")], vec![kbd("a"), kbd("b")]);
        assert!(state.lookup(&mut vec![].into()).is_err());
        assert!(state.lookup(&mut vec![kbd("a")].into()).is_err());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a"), kbd("b")].into()).unwrap(),
            &command
        ));
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("b")].into()).unwrap(),
            &command
        ));
    }

    #[test]
    fn lookup_multiple_modes() {
        let mut state = State::new(DebugRenderer::new());
        let command_mode = blank_command();
        let command_global = blank_command();
        let mode = Mode::new();
        mode.key_map
            .lock()
            .bind(vec![kbd("a")], command_mode.clone());
        state
            .default_key_map
            .lock()
            .bind(vec![kbd("a")], command_global.clone());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a")].into()).unwrap(),
            &command_global
        ));
        state.global_modes.push(Arc::new(Mutex::new(mode)));
        assert!(state.lookup(&mut vec![].into()).is_err());
        assert!(Arc::ptr_eq(
            &state.lookup(&mut vec![kbd("a")].into()).unwrap(),
            &command_mode
        ));
    }

    #[test]
    fn lookup_get_insert() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::new())));
        let insert = state.lock().lookup(&mut vec![kbd("a")].into()).unwrap();
        insert.execute(state.clone()).unwrap();
        let buffer = state.lock().display.selected_window_buffer();
        assert_eq!(buffer.lock().to_string(), "a");
    }

    #[test]
    fn is_displayable_newline() {
        assert!(is_displayable('\n'));
    }

    #[test]
    fn is_displayable_ascii() {
        assert!(is_displayable('a'));
        assert!(is_displayable('!'));
    }

    #[test]
    fn is_not_displayable_feed() {
        assert!(!is_displayable('\r'));
    }

    #[test]
    fn is_displayable_space() {
        assert!(is_displayable(' '));
    }

    #[test]
    fn is_displayable_tab() {
        assert!(is_displayable('\t'));
    }
}
