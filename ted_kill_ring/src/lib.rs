#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate ted_core;
extern crate ted_mark;

use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex;
use ted_core::*;
use ted_mark::*;

#[derive(PartialEq, Eq, Hash)]
struct W(*const Window);
unsafe impl Send for W {}
unsafe impl Sync for W {}

#[derive(Debug)]
struct KillRing {
    ring: Vec<String>,
    pos: usize,
}

lazy_static! {
    static ref KILLS: Mutex<HashMap<W, KillRing>> = Mutex::new(HashMap::new());
}

fn insert_kill(window: &Window, substring: String) {
    let mut kills = KILLS.lock();
    match kills.get_mut(&W(window)) {
        Some(kill_ring) => {
            kill_ring.ring.push(substring);
            kill_ring.pos = kill_ring.ring.len() - 1;
            return;
        },
        None => {},
    }
    kills.insert(W(window), KillRing { ring: vec![substring], pos: 0 } );
}

pub fn copy_region(window: &Window) {
    let substring = {
        let buffer = window.buffer.lock();
        substring_region(window, &buffer).1
    };
    insert_kill(window, substring);
}

pub fn kill_region(window: &Window) {
    let substring = {
        let mut buffer = window.buffer.lock();
        let (region, substring) = substring_region(window, &buffer);
        buffer.delete_region(region.start.get(), region.end.get()).unwrap();
        substring
    };
    insert_kill(window, substring);
}

pub fn paste(window: &Window) {
    let mut buffer = window.buffer.lock();
    let mut kills = KILLS.lock();
    if let Some(kill_ring) = kills.get_mut(&W(window)) {
        if !kill_ring.ring.is_empty() {
            let pos = window.cursor.clone().updated(&buffer).get();
            let s = &kill_ring.ring[kill_ring.pos];
            buffer.insert_str(pos, s).unwrap();
        }
    }
}

pub fn paste_pop(window: &Window, times: isize) {
    fn modulus(a: isize, b: isize) -> isize {
        ((a % b) + b) % b
    }
    let mut kills = KILLS.lock();
    if let Some(kill_ring) = kills.get_mut(&W(window)) {
        debug_assert!(kill_ring.ring.len() > 0);
        kill_ring.pos = modulus(kill_ring.pos as isize - times,
                                kill_ring.ring.len() as isize) as usize;
    }
}

pub fn copy_region_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let selected_window = selected_window.lock();
    copy_region(&selected_window);
    Ok(())
}

pub fn kill_region_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let mut selected_window = selected_window.lock();
    kill_region(&selected_window);
    selected_window.update_cursor();
    Ok(())
}

pub fn paste_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let mut selected_window = selected_window.lock();
    paste(&selected_window);
    selected_window.update_cursor();
    Ok(())
}

pub fn paste_pop_command(state: Arc<Mutex<State>>, _: Arc<Mutex<Display>>) -> Result<(), ()> {
    let selected_window = state.lock().selected_window.clone();
    let selected_window = selected_window.lock();
    paste_pop(&selected_window, 1);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_nothing() {
        let window = Window::new();
        paste(&window);
        assert_eq!(window.buffer.lock().to_string(), "");
    }

    #[test]
    fn paste_end() {
        let mut window = Window::new();
        window.insert_str("americaned").unwrap();
        insert_kill(&window, "abc".to_string());
        assert_eq!(window.cursor.get(), 10);

        paste(&window);
        window.update_cursor();

        assert_eq!(window.buffer.lock().to_string(), "americanedabc");
        assert_eq!(window.cursor.get(), 13);
    }

    #[test]
    fn paste_empty_buffer() {
        let mut window = Window::new();
        window.insert_str("").unwrap();
        insert_kill(&window, "abc".to_string());
        insert_kill(&window, "def".to_string());
        assert_eq!(window.cursor.get(), 0);

        paste(&window);
        window.update_cursor();

        assert_eq!(window.buffer.lock().to_string(), "def");
        assert_eq!(window.cursor.get(), 3);
    }

    #[test]
    fn paste_empty_string() {
        let mut window = Window::new();
        window.insert_str("").unwrap();
        insert_kill(&window, "abc".to_string());
        insert_kill(&window, "def".to_string());
        assert_eq!(window.cursor.get(), 0);

        paste(&window);
        window.update_cursor();

        assert_eq!(window.buffer.lock().to_string(), "def");
        assert_eq!(window.cursor.get(), 3);
    }
}
