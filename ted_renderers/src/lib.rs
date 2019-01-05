extern crate gtk;
extern crate gdk;
extern crate pancurses_result;
extern crate parking_lot;
extern crate ted_core;

mod curses_renderer;
pub use curses_renderer::*;

mod gtk_renderer;
pub use gtk_renderer::*;
