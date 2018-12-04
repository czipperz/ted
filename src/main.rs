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

use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;
use ted_user_cfg::*;

fn main() {
    let mut state = State::new();
    setup_state(&mut state);
    let mut display = CursesDisplay::new().unwrap();
    display.show(&state).unwrap();
    main_loop(Arc::new(Mutex::new(state)), Arc::new(Mutex::new(display))).unwrap_err();
}

fn main_loop(state: Arc<Mutex<State>>, display: Arc<Mutex<Display>>) -> Result<(), ()> {
    loop {
        increment(state.clone(), display.clone())?;
    }
}

fn increment(state: Arc<Mutex<State>>, display: Arc<Mutex<Display>>) -> Result<(), ()> {
    fn increment_(state: Arc<Mutex<State>>, display: Arc<Mutex<Display>>,
                  inputs: &mut VecDeque<Input>) -> Result<(), ()> {
        match { let mut display = display.lock(); display.getch() } {
            Some(input) => {
                inputs.push_back(input);
                match { let state = state.lock(); KeyMap::lookup(&state.default_key_map, inputs, true) } {
                    Ok(action) => {
                        let r = action(state.clone(), display.clone());
                        let state = state.lock();
                        display.lock().show(&state).unwrap();
                        r
                    },
                    Err(true) => {
                        increment_(state, display, inputs)
                    },
                    Err(false) => {
                        match input {
                            Input::Key { key, control: false, alt: false, function: false }
                            if !key.is_control() => {
                                {
                                    let window = state.lock().selected_window.clone();
                                    let mut window = window.lock();
                                    window.insert(key)?;
                                }
                                let state = state.lock();
                                display.lock().show(&state)?;
                            },
                            _ => {
                                log(format!("Invalid input {:?}", input));
                            },
                        }
                        Ok(())
                    },
                }
            },
            None => Ok(()),
        }
    }

    increment_(state, display, &mut VecDeque::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn increment_1() {
        let display = Arc::new(Mutex::new(DebugDisplay::new(
            vec![kbd!('a'), kbd!('b'), kbd!('c'), kbd!('q')])));
        let state = Arc::new(Mutex::new(State::new()));
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
        }
        increment(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "a");
        }
        assert_eq!(display.lock().buffer,
                   vec!["a                   ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        increment(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ab");
        }
        assert_eq!(display.lock().buffer,
                   vec!["ab                  ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        increment(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 3);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abc");
        }
        assert_eq!(display.lock().buffer,
                   vec!["abc                 ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        increment(state.clone(), display.clone()).unwrap();
        {
            let state = state.lock();
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcq");
        }
        assert_eq!(display.lock().buffer,
                   vec!["abcq                ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        {
            let state = state.lock();
            let mut default_keyset = state.default_key_map.lock();
            default_keyset.bind(vec![kbd!('q')], Arc::new(|_, _| Err(())));
        }
        display.lock().inputs.push_back(kbd!('q'));
        increment(state.clone(), display.clone()).unwrap_err();

        { let state = state.lock(); display.lock().show(&state).unwrap(); }
        assert_eq!(display.lock().buffer,
                   vec!["abcq                ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }
}
