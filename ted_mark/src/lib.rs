#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate ted_core;

use std::collections::HashMap;
use std::ops::Range;
use parking_lot::Mutex;
use ted_core::*;

#[derive(PartialEq, Eq, Hash)]
struct W(*const Window);
unsafe impl Send for W {}
unsafe impl Sync for W {}

lazy_static! {
    static ref MARKS: Mutex<HashMap<W, Cursor>> = Mutex::new(HashMap::new());
}

pub fn is_mark_active(window: &Window) -> bool {
    let marks = MARKS.lock();
    marks.contains_key(&W(window))
}

pub fn get_mark(window: &Window) -> Option<Cursor> {
    let marks = MARKS.lock();
    marks.get(&W(window)).cloned()
}

pub fn get_region(window: &Window) -> Range<Cursor> {
    debug_assert!(is_mark_active(window));
    match get_mark(window) {
        Some(mark) => {
            let cursor = window.cursor.clone();
            if mark.get() < cursor.get() {
                mark..cursor
            } else {
                cursor..mark
            }
        },
        None => {
            window.cursor.clone()..window.cursor.clone()
        },
    }
}

pub fn substring_region(window: &Window, buffer: &Buffer) -> (Range<Cursor>, String) {
    let mut region = get_region(window);
    region.start.update(&buffer);
    region.end.update(&buffer);
    let substring = buffer.substring(region.start.get(), region.end.get()).unwrap();
    (region, substring)
}

pub fn set_mark(window: &Window, cursor: Cursor) {
    let mut marks = MARKS.lock();
    marks.insert(W(window), cursor);
}

pub fn remove_mark(window: &Window) {
    let mut marks = MARKS.lock();
    marks.remove(&W(window));
}

pub fn set_mark_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    let selected_window = state.selected_window.lock();
    set_mark(&selected_window, selected_window.cursor.clone());
    Ok(())
}

pub fn remove_mark_command(state: &mut State, _: &mut Display) -> Result<(), ()> {
    let selected_window = state.selected_window.lock();
    remove_mark(&selected_window);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_mark_with_set_is_correct() {
        let window = Window::new();
        let cursor = Cursor::new();
        set_mark(&window, cursor);
        assert_eq!(get_mark(&window).unwrap().get(), 0);
        assert!(is_mark_active(&window));
    }

    #[test]
    fn get_mark_without_set_is_none() {
        let window = Window::new();
        assert!(get_mark(&window).is_none());
        assert!(!is_mark_active(&window));
    }

    #[test]
    fn remove_mark_works() {
        let window = Window::new();
        let cursor = Cursor::new();
        set_mark(&window, cursor);
        assert!(get_mark(&window).is_some());
        assert!(is_mark_active(&window));
        remove_mark(&window);
        assert!(get_mark(&window).is_none());
        assert!(!is_mark_active(&window));
    }
}
