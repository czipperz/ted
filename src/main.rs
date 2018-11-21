extern crate parking_lot;
extern crate ted_core;
extern crate ted_common_commands;

use std::sync::Arc;
use parking_lot::Mutex;
use ted_core::*;
use ted_common_commands::*;

fn setup_state(state: &mut State) {
    let mut default_key_map = state.default_key_map.lock();
    default_key_map.bind(kbd!(BACKSPACE), KeyBind::Action(Arc::new(backspace_command)));
    default_key_map.bind(kbd!(C-'b'), KeyBind::Action(Arc::new(backward_char_command)));
    default_key_map.bind(kbd!(C-'f'), KeyBind::Action(Arc::new(forward_char_command)));
    default_key_map.bind(kbd!(A-'b'), KeyBind::Action(Arc::new(backward_word_command)));
    default_key_map.bind(kbd!(A-'f'), KeyBind::Action(Arc::new(forward_word_command)));
    default_key_map.bind(kbd!(C-'a'), KeyBind::Action(Arc::new(begin_of_line_command)));
    default_key_map.bind(kbd!(C-'e'), KeyBind::Action(Arc::new(end_of_line_command)));
    default_key_map.bind(kbd!(C-'z'), KeyBind::Action(Arc::new(undo_command)));
    default_key_map.bind(kbd!(A-'z'), KeyBind::Action(Arc::new(redo_command)));
    default_key_map.bind(kbd!(C-'n'), KeyBind::Action(Arc::new(forward_line_command)));
    default_key_map.bind(kbd!(C-'p'), KeyBind::Action(Arc::new(backward_line_command)));

    default_key_map.bind(Input::Key { c: 'x', control: true, alt: false },
                         KeyBind::SubMap(Arc::new(Mutex::new({
                             let mut cx = KeyMap::default();
                             cx.bind(kbd!(C-'c'), KeyBind::Action(Arc::new(|_, _| Err(()))));
                             cx.bind(kbd!('1'), KeyBind::Action(Arc::new(close_other_windows_command)));
                             cx.bind(kbd!('2'), KeyBind::Action(Arc::new(horizontal_split_command)));
                             cx.bind(kbd!('3'), KeyBind::Action(Arc::new(vertical_split_command)));
                             cx.bind(kbd!('0'), KeyBind::Action(Arc::new(close_window_command)));
                             cx.bind(kbd!('o'), KeyBind::Action(Arc::new(other_window_clockwise_command)));
                             cx.bind(kbd!(C-'o'), KeyBind::Action(Arc::new(other_window_counter_clockwise_command)));
                             cx
                         }))));
}

fn main() {
    let mut display = CursesDisplay::new().unwrap();
    let mut state = State::new();
    setup_state(&mut state);
    main_loop(&mut state, &mut display).unwrap_err();
}

fn main_loop(state: &mut State, display: &mut Display) -> Result<(), ()> {
    loop {
        increment(state, display)?;
    }
}

fn increment(state: &mut State, display: &mut Display) -> Result<(), ()> {
    let key_map = state.default_key_map.clone();
    let ch = display.getch();
    increment_(state, &key_map, false, display, ch)
}

fn increment_(state: &mut State, key_map: &Arc<Mutex<KeyMap>>, recursed: bool,
              display: &mut Display, ch: Option<Input>) -> Result<(), ()> {
    match ch {
        Some(input) => {
            let x = {
                let key_map = key_map.lock();
                let l = key_map.lookup(&input);
                l.cloned()
            };
            match x {
                Some(KeyBind::Action(f)) => {
                    let r = f(state, display);
                    display.show(&state).unwrap();
                    r
                },
                Some(KeyBind::SubMap(sub_map)) => {
                    let ch = display.getch();
                    increment_(state, &sub_map, true, display, ch)
                },
                None => {
                    if !recursed {
                        match input {
                            Input::Key { c, control: false, alt: false } => {
                                {
                                    let mut selected_window = state.selected_window.lock();
                                    selected_window.insert(c).unwrap();
                                }
                                display.show(&state).unwrap();
                            },
                            _ => {},
                        }
                    }
                    Ok(())
                },
            }
        },
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn main_loop_immediate_quit() {
        let mut display = DebugDisplay::new(vec![
            Input::Key { c: 'q', control: false, alt: false }]);
        let mut state = State::new();
        {
            let mut default_keyset = state.default_key_map.lock();
            default_keyset.bind(Input::Key { c: 'q', control: false, alt: false },
                                KeyBind::Action(Arc::new(|_, _| Err(()))));
        }
        main_loop(&mut state, &mut display).unwrap_err();
        assert_eq!(display.buffer,
                   vec!["                    ".chars().collect::<Vec<_>>(),
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
                        "                    ".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn increment_1() {
        let mut display = DebugDisplay::new(vec![
            Input::Key { c: 'a', control: false, alt: false },
            Input::Key { c: 'b', control: false, alt: false },
            Input::Key { c: 'c', control: false, alt: false },
            Input::Key { c: 'q', control: false, alt: false },
        ]);
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
                        "                    ".chars().collect::<Vec<_>>()]);

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
                        "                    ".chars().collect::<Vec<_>>()]);

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
                        "                    ".chars().collect::<Vec<_>>()]);

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
                        "                    ".chars().collect::<Vec<_>>()]);

        {
            let mut default_keyset = state.default_key_map.lock();
            default_keyset.bind(Input::Key { c: 'q', control: false, alt: false },
                                KeyBind::Action(Arc::new(|_, _| Err(()))));
        }
        display.inputs.push_back(Input::Key { c: 'q', control: false, alt: false });
        increment(&mut state, &mut display).unwrap_err();
    }

    #[test]
    fn increment_vertical_split() {
        let mut state = State::new();
        let mut display = DebugDisplay::new(vec![
            Input::Key { c: 'a', control: false, alt: false },
            Input::Key { c: 'x', control: true, alt: false },
            Input::Key { c: '3', control: false, alt: false },
            Input::Key { c: 'b', control: false, alt: false },
        ]);

        {
            let mut default_key_map = state.default_key_map.lock();
            default_key_map.bind(Input::Key { c: 'x', control: true, alt: false },
                                 KeyBind::SubMap(Arc::new(Mutex::new({
                                     let mut cx = KeyMap::default();
                                     cx.bind(kbd!('3'), KeyBind::Action(Arc::new(vertical_split_command)));
                                     cx
                                 }))));
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
                        "                    ".chars().collect::<Vec<_>>()]);
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
                        "          |         ".chars().collect::<Vec<_>>()]);
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
                        "          |         ".chars().collect::<Vec<_>>()]);
        assert_eq!(display.selected_cursors,
                   vec![(0, 2)]);
        assert_eq!(display.unselected_cursors,
                   vec![(0, 13)]);
    }
}
