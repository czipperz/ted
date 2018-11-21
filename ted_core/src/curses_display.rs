use pancurses;
use display::Display;
use state::State;
use input::Input;
use draw::*;
use logger::*;

/// An implementation of [`Display`] for a curses backend.
///
/// This struct is used to wrap the curses process.
/// The backend is implemented via the `pancurses` crate.
pub struct CursesDisplay {
    window: pancurses::Window,
    cursor_y: i32,
    cursor_x: i32,
    stalling_escape: bool,
}

fn check(code: i32) -> Result<(), ()> {
    if code == pancurses::ERR {
        Err(())
    } else {
        Ok(())
    }
}

impl CursesDisplay {
    /// Initialize the curses backend and wrap it in the CursesDisplay object.
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

impl Display for CursesDisplay {
    fn show(&mut self, state: &State) -> Result<(), ()> {
        let (rows, columns) = self.window.get_max_yx();
        draw(self, state, rows as usize, columns as usize)?;
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
                if cfg!(debug_assertions) {
                    log(format!("{:?} ({:?} = {:?})", ch, c, c as u32));
                }
                Some(ch)
            },
            k => {
                if k.is_some() {
                    if cfg!(debug_assertions) {
                        log(format!("{:?}", k));
                    }
                }
                None
            },
        }
    }
}

fn convert_to_key(c: char, alt: bool) -> Input {
    if c == '\n' /* 10 */ {
        Input::Key { key: c, control: false, alt }
    } else if c >= 1 as char && c <= 26 as char {
        Input::Key { key: (c as u8 - 1 + 'a' as u8) as char,
                     control: true, alt }
    /*} else if c >= 27 as char && c <= 31 as char {
        Input::Key { c: (c as u8 - 27 + '1' as u8) as char,
                     control: true, alt: false }*/
    } else {
        Input::Key { key: c, control: false, alt }
    }
}

impl DisplayDraw for CursesDisplay {
    fn erase(&mut self) -> Result<(), ()> {
        check(self.window.erase())
    }

    fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), ()> {
        let c =
            match ch {
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
            },
            Attribute::UnselectedCursor => Ok(()),
            //check(self.window.chgat(1, pancurses::Attribute::Reverse.into(), 0)) ,
        }
    }
}

/// Uninitialize the curses backend.
impl Drop for CursesDisplay {
    fn drop(&mut self) {
        pancurses::endwin();
        if cfg!(debug_assertions) {
            print_log();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_key_1() {
        assert_ne!(convert_to_key(2 as char, true),
                   Input::Key { key: 'a', control: true, alt: true });
        assert_eq!(convert_to_key(1 as char, true),
                   Input::Key { key: 'a', control: true, alt: true });
        assert_eq!(convert_to_key(2 as char, false),
                   Input::Key { key: 'b', control: true, alt: false });
    }
}
