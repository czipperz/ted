//! This crate implements the core functionality of ted.
//!
//! The central components of the text editor are implemented here.
//!
//! A high level overview is that the [`Display`] produces events.
//! These events cause certain commands to be triggered.  The command
//! then executes, modifying the [`State`].  Then the [`Display`] is
//! reshown and the process restarts.
//!
//! [`Display`]: trait.Display.html
//! [`State`]: struct.State.html

#[macro_use]
extern crate lazy_static;
extern crate pancurses;
extern crate parking_lot;
extern crate by_address;

#[macro_use]
mod input;
pub use input::*;
mod curses_display;
pub use curses_display::*;
mod debug_display;
pub use debug_display::*;
mod display;
pub use display::*;
mod key_map;
pub use key_map::*;
mod layout;
pub use layout::*;
mod state;
pub use state::*;
mod window;
pub use window::*;
mod logger;
pub use logger::*;
mod buffer;
pub use buffer::Buffer;

// private modules
mod draw;
mod buffer_contents;
mod cursor;
mod change;
