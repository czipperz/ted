use draw::*;
use input::Input;
use layout::Layout;
use logger::*;
use pancurses;
use parking_lot::Mutex;
use renderer::Renderer;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use window::Window;

/// An implementation of [`Renderer`] for a curses backend.
///
/// This struct is used to wrap the curses process.
/// The backend is implemented via the `pancurses` crate.
pub struct CursesRenderer {
    window: pancurses::Window,
    cursor_y: i32,
    cursor_x: i32,
    stalling_escape: bool,
}

unsafe impl Send for CursesRenderer {}

fn check(code: i32) -> Result<(), ()> {
    if code == pancurses::ERR {
        Err(())
    } else {
        Ok(())
    }
}

impl CursesRenderer {
    /// Initialize the curses backend and wrap it in the CursesRenderer object.
    pub fn new() -> Result<Self, ()> {
        let window = pancurses::initscr();
        check(pancurses::raw())?;
        check(pancurses::noecho())?;
        Ok(Self {
            window,
            cursor_y: 0,
            cursor_x: 0,
            stalling_escape: false,
        })
    }
}

impl Renderer for CursesRenderer {
    fn show(&mut self, layout: &Layout, selected_window: Option<&Arc<Mutex<Window>>>,
            messages: &mut VecDeque<String>, message_display_time: &mut Option<Instant>) -> Result<(), ()> {
        let (rows, columns) = self.window.get_max_yx();
        let rows = rows as usize;
        let columns = columns as usize;
        draw(self, layout, selected_window, rows, columns)?;
        match message_display_time.clone() {
            Some(time) => {
                if time.elapsed() > Duration::from_secs(10) {
                    messages.pop_front();
                    *message_display_time = None;
                }
            }
            None => {
                if !messages.is_empty() {
                    *message_display_time = Some(Instant::now());
                }
            },
        }
        match messages.front() {
            Some(message) => {
                self.set_attribute(rows - 3, 9, Attribute::Inverted)?;
                self.putch(rows - 3, 9, Character::Character('.'))?;
                self.set_attribute(rows - 2, 9, Attribute::Inverted)?;
                self.putch(rows - 2, 9, Character::VLine)?;
                for i in 10..=columns-10 {
                    self.set_attribute(rows - 3, i, Attribute::Inverted)?;
                    self.putch(rows - 3, i, Character::HLine)?;
                    self.set_attribute(rows - 2, i, Attribute::Inverted)?;
                    self.putch(rows - 2, i, Character::Character(' '))?
                }
                self.set_attribute(rows - 3, columns - 9, Attribute::Inverted)?;
                self.putch(rows - 3, columns - 9, Character::Character('.'))?;
                self.set_attribute(rows - 2, columns - 9, Attribute::Inverted)?;
                self.putch(rows - 2, columns - 9, Character::VLine)?;
                draw_window(
                    self,
                    message.chars(),
                    false,
                    None,
                    None,
                    rows - 2,
                    10,
                    1,
                    columns - 20,
                )?;
            },
            None => (),
        }
        check(self.window.mv(self.cursor_y as i32, self.cursor_x as i32))?;
        check(self.window.refresh())?;
        Ok(())
    }

    fn getch(&mut self) -> Option<Input> {
        match self.window.getch() {
            Some(pancurses::Input::Character(c)) if c == 27 as char => {
                self.stalling_escape = true;
                self.getch()
            }
            Some(pancurses::Input::Character(c)) => {
                let ch = convert_to_key(c, self.stalling_escape);
                self.stalling_escape = false;
                log_debug(format!(
                    "{:?} ({:?} = {:?}) {:?}",
                    ch,
                    c,
                    c as u32,
                    pancurses::keyname(c as i32)
                ));
                Some(ch)
            }
            k => {
                if k.is_some() {
                    log_debug(format!("{:?}", k));
                }
                None
            }
        }
    }
}

fn convert_to_key(c: char, alt: bool) -> Input {
    if c == '\n'
    /* 10 */
    {
        Input::Key {
            key: c,
            control: false,
            alt,
            function: false,
        }
    } else if c == 0 as char
    /* 0 */
    {
        Input::Key {
            key: '@',
            control: true,
            alt,
            function: false,
        }
    } else if c >= 1 as char && c <= 26 as char {
        let key = (c as u8 - 1 + 'a' as u8) as char;
        Input::Key {
            key,
            control: true,
            alt,
            function: false,
        }
    } else {
        Input::Key {
            key: c,
            control: false,
            alt,
            function: false,
        }
    }
}

impl DrawableRenderer for CursesRenderer {
    fn erase(&mut self) -> Result<(), ()> {
        check(self.window.erase())
    }

    fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), ()> {
        let c = match ch {
            Character::Character(ch) => ch,
            Character::VLine => '|',
            Character::HLine => '-',
        };
        check(self.window.mvaddch(y as i32, x as i32, c))
    }

    fn set_attribute(&mut self, y: usize, x: usize, at: Attribute) -> Result<(), ()> {
        match at {
            Attribute::SelectedCursor => {
                self.cursor_y = y as i32;
                self.cursor_x = x as i32;
                Ok(())
            }
            Attribute::UnselectedCursor => Ok(()),
            Attribute::Inverted => check(self.window.mvchgat(
                y as i32,
                x as i32,
                1,
                pancurses::Attribute::Reverse.into(),
                0,
            )),
        }
    }
}

/// Uninitialize the curses backend.
impl Drop for CursesRenderer {
    fn drop(&mut self) {
        pancurses::endwin();
        print_log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_key_1() {
        assert_ne!(convert_to_key(2 as char, true), kbd!(C - A - 'a'));
        assert_eq!(convert_to_key(1 as char, true), kbd!(C - A - 'a'));
        assert_eq!(convert_to_key(2 as char, false), kbd!(C - 'b'));
    }
}
