#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate ted_core;
extern crate ted_mark;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
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
        }
        None => {}
    }
    kills.insert(
        W(window),
        KillRing {
            ring: vec![substring],
            pos: 0,
        },
    );
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
        buffer
            .delete_region(region.start.get(), region.end.get())
            .unwrap();
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
        kill_ring.pos = modulus(
            kill_ring.pos as isize - times,
            kill_ring.ring.len() as isize,
        ) as usize;
    }
}

#[derive(Debug)]
pub struct CopyRegionCommand;

/// Construct a [`CopyRegionCommand`].
///
/// [`CopyRegionCommand`]: struct.CopyRegionCommand.html
pub fn copy_region_command() -> Arc<CopyRegionCommand> {
    Arc::new(CopyRegionCommand)
}

impl Command for CopyRegionCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        copy_region(&selected_window);
        Ok(())
    }
}

#[derive(Debug)]
pub struct KillRegionCommand;

/// Construct a [`KillRegionCommand`].
///
/// [`KillRegionCommand`]: struct.KillRegionCommand.html
pub fn kill_region_command() -> Arc<KillRegionCommand> {
    Arc::new(KillRegionCommand)
}

impl Command for KillRegionCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        kill_region(&selected_window);
        selected_window.update_cursor();
        Ok(())
    }
}

#[derive(Debug)]
pub struct PasteCommand;

/// Construct a [`PasteCommand`].
///
/// [`PasteCommand`]: struct.PasteCommand.html
pub fn paste_command() -> Arc<PasteCommand> {
    Arc::new(PasteCommand)
}

impl Command for PasteCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        paste(&selected_window);
        selected_window.update_cursor();
        Ok(())
    }
}

#[derive(Debug)]
pub struct PastePopCommand;

/// Construct a [`PastePopCommand`].
///
/// [`PastePopCommand`]: struct.PastePopCommand.html
pub fn paste_pop_command() -> Arc<PastePopCommand> {
    Arc::new(PastePopCommand)
}

impl Command for PastePopCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), ()> {
        let selected_window = state.lock().display.selected_window();
        let selected_window = selected_window.lock();
        paste_pop(&selected_window, 1);
        Ok(())
    }
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
