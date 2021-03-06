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
extern crate ted_common_commands;
extern crate ted_core;
extern crate ted_renderers;
extern crate ted_user_cfg;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use ted_core::*;
use ted_renderers::*;
use ted_user_cfg::*;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), String> {
    std::panic::set_hook(Box::new(|info| {
        log(format!("{}", info));
    }));
    let mut state = State::new(CursesRenderer::new().unwrap());
    setup_state(&mut state)?;
    let state = Arc::new(Mutex::new(state));
    parse_parameters(&*state)?;
    state.lock().display.show().unwrap();
    main_loop(state);
    Ok(())
}

fn parse_parameters(state: &Mutex<State>) -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        Ok(())
    } else if args.len() == 2 {
        use std::path::PathBuf;
        ted_common_commands::open_file_inplace(state, &PathBuf::from(&args[1]))
    } else {
        Err("Error: Too many arguments".to_string())
    }
}

fn main_loop(state: Arc<Mutex<State>>) {
    loop {
        match increment(state.clone()) {
            Ok(()) => (),
            Err(message) => {
                if ted_common_commands::was_closed_successfully() {
                    return;
                } else {
                    let mut state = state.lock();
                    state.display.selected_frame.lock().messages.add(message);
                    state.display.show().unwrap();
                }
            }
        }
    }
}

fn increment(state: Arc<Mutex<State>>) -> Result<(), String> {
    fn increment_(state: Arc<Mutex<State>>, inputs: &mut VecDeque<Input>) -> Result<(), String> {
        loop {
            match {
                let state = state.lock();
                state.display.getch()
            } {
                Some(input) => {
                    state.lock().display.show()?;
                    inputs.push_back(input);
                    match {
                        let state = state.lock();
                        state.lookup(inputs)
                    } {
                        Ok(action) => {
                            let r = action.execute(state.clone());
                            state.lock().display.show().unwrap();
                            return r;
                        }
                        Err(Ok(())) => (),
                        Err(Err(())) => return Ok(()),
                    }
                }
                None => state.lock().display.show()?,
            }
        }
    }

    increment_(state, &mut VecDeque::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use ted_common_commands::close_ted_command;

    #[test]
    fn increment_1() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::from(vec![
            kbd("a"),
            kbd("b"),
            kbd("c"),
            kbd("q"),
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
            default_key_map.bind(vec![kbd("q")], close_ted_command());
        }
        {
            let state = state.lock();
            unsafe { state.display.debug_renderer() }
                .inputs
                .push_back(kbd("q"));
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
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::from(vec![kbd("\n")]))));
        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);

            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "\n");
        }
    }

    #[test]
    fn increment_double_bind() {
        let state = Arc::new(Mutex::new(State::new(DebugRenderer::from(vec![
            kbd("a"),
            kbd("a"),
            kbd("b"),
        ]))));
        {
            let default_key_map = state.lock().default_key_map.clone();
            default_key_map
                .lock()
                .bind(vec![kbd("a"), kbd("b")], close_ted_command());
        }

        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();

            let buffer = selected_window.buffer.lock();
            assert_eq!(buffer.to_string(), "");

            assert_eq!(selected_window.cursor.get(), 0);
        }

        increment(state.clone()).unwrap();
        {
            let selected_window = state.lock().display.selected_window();
            let selected_window = selected_window.lock();

            let buffer = selected_window.buffer.lock();
            assert_eq!(buffer.to_string(), "b");

            assert_eq!(selected_window.cursor.get(), 1);
        }
    }
}
