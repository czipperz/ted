//! A modular thread-safe text editor written in Rust.
//!
//! This crate runs the main loop.  That is it handles which Display
//! is used (terminal vs gui) and processing events.
//!
//! Key binds are set up in the [`ted_user_cfg`] crate.
//!
//! To look at the overview of the software model, see the crate [`ted_core`].
//!
//! To look at some common commands that you can run, see the crate [`ted_common_commands`].
//!
//! To see how to create your own commands, see the documentation for [`Action`]s.
//!
//! [`ted_core`]: ../ted_core/index.html
//! [`ted_common_commands`]: ../ted_common_commands/index.html
//! [`ted_user_cfg`]: ../ted_user_cfg/index.html
//! [`Action`]: ../ted_core/type.Action.html

extern crate parking_lot;
extern crate ted_core;
extern crate ted_user_cfg;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use ted_core::*;
use ted_user_cfg::*;

fn main() {
    let mut state = State::new(CursesRenderer::new().unwrap());
    setup_state(&mut state);
    state.display.show().unwrap();
    main_loop(Arc::new(Mutex::new(state))).unwrap_err();
}

fn main_loop(state: Arc<Mutex<State>>) -> Result<(), ()> {
    loop {
        increment(state.clone())?;
    }
}

fn increment(state: Arc<Mutex<State>>) -> Result<(), ()> {
    fn increment_(state: Arc<Mutex<State>>, inputs: &mut VecDeque<Input>) -> Result<(), ()> {
        match {
            let state = state.lock();
            state.display.getch()
        } {
            Some(input) => {
                inputs.push_back(input);
                match {
                    let state = state.lock();
                    state.lookup(inputs)
                } {
                    Ok(action) => {
                        let r = action.execute(state.clone());
                        state.lock().display.show().unwrap();
                        r
                    }
                    Err(Ok(())) => increment_(state, inputs),
                    Err(Err(())) => Ok(()),
                }
            }
            None => Ok(()),
        }
    }

    increment_(state, &mut VecDeque::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[derive(Debug)]
    pub struct CloseTedCommand;

    pub fn close_ted_command() -> Arc<CloseTedCommand> {
        Arc::new(CloseTedCommand)
    }

    impl Command for CloseTedCommand {
        fn execute(&self, _: Arc<Mutex<State>>) -> Result<(), ()> {
            Err(())
        }
    }

    #[test]
    fn increment_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::from(vec![
            kbd!('a'),
            kbd!('b'),
            kbd!('c'),
            kbd!('q'),
        ]))));
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
        }
        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "a");
        }
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "a                   ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }

        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ab");
        }
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "ab                  ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }

        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 3);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abc");
        }
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abc                 ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }

        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcq");
        }
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcq                ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }

        {
            let default_key_map = state.lock().default_key_map.clone();
            let mut default_key_map = default_key_map.lock();
            default_key_map.bind(vec![kbd!('q')], close_ted_command());
        }
        {
            let state = state.lock();
            unsafe { state.display.debug_renderer() }
                .inputs
                .push_back(kbd!('q'));
        }
        increment(state.clone()).unwrap_err();

        state.lock().display.show().unwrap();
        {
            let state = state.lock();
            assert_eq!(
                unsafe { state.display.debug_renderer() }.buffer,
                vec![
                    "abcq                ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "                    ".chars().collect::<Vec<_>>(),
                    "*scratch*           ".chars().collect::<Vec<_>>()
                ]
            );
        }
    }

    #[test]
    fn increment_new_line() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::from(vec![kbd!(
            '\n'
        )]))));
        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "\n");
        }
    }
}
