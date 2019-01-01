use pancurses_result as pancurses;
use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::draw::*;
use ted_core::*;

/// An implementation of [`Renderer`] for a curses backend.
///
/// This struct is used to wrap the curses process.
/// The backend is implemented via the `pancurses` crate.
pub struct CursesRenderer {
    _print_log_on_destruction: PrintLogOnDestruction,
    curses: pancurses::Curses,
    cursor: pancurses::Point,
    stalling_escape: bool,
}

impl CursesRenderer {
    /// Initialize the curses backend and wrap it in the CursesRenderer object.
    pub fn new() -> Result<Self, ()> {
        let mut curses = pancurses::initscr()?;
        curses.set_input_buffering_mode(pancurses::InputBufferingMode::UnbufferedWithSignals)?;
        curses.set_echo_input(false)?;
        curses.window_mut().set_block_on_read(false)?;
        Ok(Self {
            _print_log_on_destruction: PrintLogOnDestruction,
            curses,
            cursor: (0, 0).into(),
            stalling_escape: false,
        })
    }
}

impl Renderer for CursesRenderer {
    fn show(
        &mut self,
        layout: &Layout,
        selected_window: Option<&Arc<Mutex<Window>>>,
        message: Option<&str>,
    ) -> Result<(), String> {
        let (rows, columns) = self.curses.window().size().into();
        let rows = rows as usize;
        let columns = columns as usize;
        draw(self, layout, selected_window, rows, columns)?;
        if let Some(message) = message {
            self.set_attribute(rows - 3, 9, Attribute::Inverted)?;
            self.putch(rows - 3, 9, Character::Character('.'))?;
            self.set_attribute(rows - 2, 9, Attribute::Inverted)?;
            self.putch(rows - 2, 9, Character::VLine)?;
            for i in 10..=columns - 10 {
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
        }
        self.curses
            .window_mut()
            .move_to(self.cursor)
            .map_err(|()| "Error: Curses mv()".to_string())?;
        self.curses
            .window_mut()
            .refresh()
            .map_err(|()| "Error: Curses refresh()".to_string())?;
        Ok(())
    }

    fn getch(&mut self) -> Option<Input> {
        match self.curses.window_mut().read_char() {
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
                    self.curses.key_name(c as i32)
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
    if c == '\n' {
        Input {
            control: false,
            alt,
            key: Key::Key(c),
        }
    } else if c == 0 as char {
        Input {
            control: true,
            alt,
            key: Key::Key('@'),
        }
    } else if c >= 1 as char && c <= 26 as char {
        Input {
            control: true,
            alt,
            key: Key::Key((c as u8 - 1 + 'a' as u8) as char),
        }
    } else {
        Input {
            control: false,
            alt,
            key: Key::Key(c),
        }
    }
}

impl DrawableRenderer for CursesRenderer {
    fn erase(&mut self) -> Result<(), String> {
        self.curses
            .window_mut()
            .erase()
            .map_err(|()| "Error: Curses erase()".to_string())
    }

    fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), String> {
        let c = match ch {
            Character::Character(ch) => ch,
            Character::VLine => '|',
            Character::HLine => '-',
        };
        self.curses
            .window_mut()
            .move_put_char((y as i32, x as i32), c)
            .map_err(|()| "Error: Curses putch()".to_string())
    }

    fn set_attribute(&mut self, y: usize, x: usize, at: Attribute) -> Result<(), String> {
        match at {
            Attribute::SelectedCursor => {
                self.cursor = (y as i32, x as i32).into();
                Ok(())
            }
            Attribute::UnselectedCursor => Ok(()),
            Attribute::Inverted => self
                .curses
                .window_mut()
                .move_change_attributes((y as i32, x as i32), 1, pancurses::Attribute::Reverse, 0)
                .map_err(|()| "Error: Curses mvchgat()".to_string()),
        }
    }
}

struct PrintLogOnDestruction;
impl Drop for PrintLogOnDestruction {
    fn drop(&mut self) {
        print_log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_key_1() {
        assert_ne!(convert_to_key(2 as char, true), kbd("C-A-a"));
        assert_eq!(convert_to_key(1 as char, true), kbd("C-A-a"));
        assert_eq!(convert_to_key(2 as char, false), kbd("C-b"));
    }
}
