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

fn close_ted_command(_: &mut State, _: &mut Display) -> Result<(), ()> {
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
    main_loop(&mut state, &mut display).unwrap_err();
}

fn main_loop(state: &mut State, display: &mut Display) -> Result<(), ()> {
    loop {
        increment(state, display)?;
    }
}

fn increment(state: &mut State, display: &mut Display) -> Result<(), ()> {
    fn increment_(state: &mut State, display: &mut Display,
                  inputs: &mut VecDeque<Input>) -> Result<(), ()> {
        match display.getch() {
            Some(input) => {
                inputs.push_back(input);
                match KeyMap::lookup(&state.default_key_map, inputs, true) {
                    Ok(action) => {
                        let r = action(state, display);
                        display.show(&state).unwrap();
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
                                    let mut window = state.selected_window.lock();
                                    window.insert(key)?;
                                }
                                display.show(state)?;
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
        let mut display = DebugDisplay::new(vec![kbd!('a'), kbd!('b'), kbd!('c'), kbd!('q')]);
        let mut state = State::new();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 0);
        }
        increment(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 1);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "a");
        }
        assert_eq!(display.buffer,
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

        increment(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 2);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "ab");
        }
        assert_eq!(display.buffer,
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

        increment(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 3);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abc");
        }
        assert_eq!(display.buffer,
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

        increment(&mut state, &mut display).unwrap();
        {
            let selected_window = state.selected_window.lock();
            assert_eq!(selected_window.cursor.get(), 4);
            let buffer = selected_window.buffer.lock();
            assert_eq!(format!("{}", *buffer), "abcq");
        }
        assert_eq!(display.buffer,
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
            let mut default_keyset = state.default_key_map.lock();
            default_keyset.bind(vec![kbd!('q')], Arc::new(|_, _| Err(())));
        }
        display.inputs.push_back(kbd!('q'));
        increment(&mut state, &mut display).unwrap_err();

        display.show(&state).unwrap();
        assert_eq!(display.buffer,
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
        let mut state = State::new();
        let mut display = DebugDisplay::new(vec![kbd!('a'), kbd!(C-'x'), kbd!('3'), kbd!('b')]);

        {
            let mut default_key_map = state.default_key_map.lock();
            default_key_map.bind(vec![kbd!(C-'x'), kbd!('3')], Arc::new(vertical_split_command));
        }

        increment(&mut state, &mut display).unwrap();
        assert_eq!(display.buffer,
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
        assert_eq!(display.selected_cursors,
                   vec![(0, 1)]);

        increment(&mut state, &mut display).unwrap();
        assert_eq!(display.buffer,
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
        assert_eq!(display.selected_cursors,
                   vec![(0, 1)]);
        assert_eq!(display.unselected_cursors,
                   vec![(0, 12)]);

        increment(&mut state, &mut display).unwrap();
        assert_eq!(display.buffer,
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
        assert_eq!(display.selected_cursors,
                   vec![(0, 2)]);
        assert_eq!(display.unselected_cursors,
                   vec![(0, 13)]);
    }
}
