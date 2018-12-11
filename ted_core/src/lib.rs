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
extern crate by_address;
extern crate pancurses;
extern crate parking_lot;

#[macro_use]
mod input;
pub use input::*;
mod display;
pub use display::*;
mod frame;
pub use frame::*;
mod renderer;
pub use renderer::*;
mod curses_renderer;
pub use curses_renderer::*;
mod debug_renderer;
pub use debug_renderer::*;
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
pub use buffer::{Buffer, BufferName};
mod cursor;
pub use cursor::*;
mod mode;
pub use mode::*;

// private modules
mod buffer_contents;
mod change;
mod draw;
