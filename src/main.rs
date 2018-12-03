//! A modular thread-safe text editor written in Rust.
//!
//! This crate runs the main loop.  That is it handles which Display
//! is used (terminal vs gui) and processing events.
//!
//! This isn't incredibly interesting.  Right now key binds are set up
//! in this crate but that will change soon to allow more user
//! customizability.
//!
//! To look at the overview of the software model, see the crate [`ted_core`].
//! To look at some common commands that you can run, see the crate [`ted_common_commands`].
//!
//! [`ted_core`]: ../ted_core/index.html
//! [`ted_common_commands`]: ../ted_common_commands/index.html

extern crate parking_lot;
extern crate ted_core;
extern crate ted_common_commands;
extern crate ted_mark;
extern crate ted_kill_ring;

use std::collections::VecDeque;
use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;
use ted_common_commands::*;
use ted_mark::*;
use ted_kill_ring::*;

fn close_ted_command(_: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    Err(())
}

fn setup_state(state: &mut State) {
    let mut default_key_map = state.default_key_map.lock();
    default_key_map.bind(vec![kbd!(C-'a')], Arc::new(begin_of_line_command));
    default_key_map.bind(vec![kbd!(C-'b')], Arc::new(backward_char_command));
    default_key_map.bind(vec![kbd!(A-'b')], Arc::new(backward_word_command));
    default_key_map.bind(vec![kbd!(C-A-'b')], Arc::new(backward_group_command));
    default_key_map.bind(vec![kbd!(C-'d')], Arc::new(delete_forward_char_command));
    default_key_map.bind(vec![kbd!(C-'e')], Arc::new(end_of_line_command));
    default_key_map.bind(vec![kbd!(C-'f')], Arc::new(forward_char_command));
    default_key_map.bind(vec![kbd!(A-'f')], Arc::new(forward_word_command));
    default_key_map.bind(vec![kbd!(C-A-'f')], Arc::new(forward_group_command));
    default_key_map.bind(vec![kbd!(C-'g')], Arc::new(remove_mark_command));
    default_key_map.bind(vec![kbd!(C-'n')], Arc::new(forward_line_command));
    default_key_map.bind(vec![kbd!(C-'p')], Arc::new(backward_line_command));
    default_key_map.bind(vec![kbd!(C-A-'u')], Arc::new(up_group_command));
    default_key_map.bind(vec![kbd!(C-'w')], Arc::new(kill_region_command));
    default_key_map.bind(vec![kbd!(A-'w')], Arc::new(copy_region_command));
    default_key_map.bind(vec![kbd!(C-'y')], Arc::new(paste_command));
    default_key_map.bind(vec![kbd!(A-'y')], Arc::new(paste_pop_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('1')], Arc::new(close_other_windows_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('2')], Arc::new(horizontal_split_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('3')], Arc::new(vertical_split_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('0')], Arc::new(close_window_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'c')], Arc::new(close_ted_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!('o')], Arc::new(other_window_clockwise_command));
    default_key_map.bind(vec![kbd!(C-'x'), kbd!(C-'o')], Arc::new(other_window_counter_clockwise_command));
    default_key_map.bind(vec![kbd!(C-'z')], Arc::new(undo_command));
    default_key_map.bind(vec![kbd!(A-'z')], Arc::new(redo_command));
    default_key_map.bind(vec![kbd!(BACKSPACE)], Arc::new(delete_backward_char_command));
    default_key_map.bind(vec![kbd!(C-'@')], Arc::new(set_mark_command));
}

fn main() {
    let mut display = CursesDisplay::new().unwrap();
    let mut state = State::new();
    setup_state(&mut state);
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

    #[test]
    fn increment_vertical_split() {
        let state = Arc::new(Mutex::new(State::new()));
        let display = Arc::new(Mutex::new(DebugDisplay::new(
            vec![kbd!('a'), kbd!(C-'x'), kbd!('3'), kbd!('b')])));

        {
            let state = state.lock();
            let mut default_key_map = state.default_key_map.lock();
            default_key_map.bind(vec![kbd!(C-'x'), kbd!('3')], Arc::new(vertical_split_command));
        }

        increment(state.clone(), display.clone()).unwrap();
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
        assert_eq!(display.lock().selected_cursors, vec![(0, 1)]);

        increment(state.clone(), display.clone()).unwrap();
        assert_eq!(display.lock().buffer,
                   vec!["a         |a        ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "*scratch* |*scratch*".chars().collect::<Vec<_>>()]);
        assert_eq!(display.lock().selected_cursors, vec![(0, 1)]);
        assert_eq!(display.lock().unselected_cursors, vec![(0, 12)]);

        increment(state.clone(), display.clone()).unwrap();
        assert_eq!(display.lock().buffer,
                   vec!["ab        |ab       ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "          |         ".chars().collect::<Vec<_>>(),
                        "*scratch* |*scratch*".chars().collect::<Vec<_>>()]);
        assert_eq!(display.lock().selected_cursors, vec![(0, 2)]);
        assert_eq!(display.lock().unselected_cursors, vec![(0, 13)]);
    }
}
