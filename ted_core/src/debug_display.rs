use std::collections::VecDeque;
use display::Display;
use input::Input;
use state::State;
use draw::*;

/// An implementation of [`Display`] for debugging.
///
/// This renders a 20 column, 15 row display via a 2D array of
/// characters and allow for normal usage as a [`Display`].
///
/// You can emulate the user typing by pushing to `inputs`.
///
/// # Examples
///
/// ```
/// # use ted_core::{DebugDisplay, Display, State};
/// let mut display = DebugDisplay::new(Vec::new());
/// assert_eq!(display.buffer,
///            vec!["                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>()]);
///
/// let state = State::new();
/// display.show(&state).unwrap();
/// assert_eq!(display.buffer,
///            vec!["                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "*scratch*           ".chars().collect::<Vec<_>>()]);
///
/// {
///     let mut selected_window = state.selected_window.lock();
///     let selected_window = &mut *selected_window;
///     let mut buffer = selected_window.buffer.lock();
///     buffer.insert_str(0, "abcd").unwrap();
/// }
/// display.show(&state).unwrap();
/// assert_eq!(display.selected_cursors, vec![(0, 4)]);
/// assert_eq!(display.unselected_cursors, vec![]);
/// assert_eq!(display.buffer,
///            vec!["abcd                ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "                    ".chars().collect::<Vec<_>>(),
///                 "*scratch*           ".chars().collect::<Vec<_>>()]);
/// ```
///
/// [`Display`]: trait.Display.html
pub struct DebugDisplay {
    pub inputs: VecDeque<Input>,
    pub buffer: Vec<Vec<char>>,
    pub selected_cursors: Vec<(usize, usize)>,
    pub unselected_cursors: Vec<(usize, usize)>,
}

impl DebugDisplay {
    pub fn new(inputs: Vec<Input>) -> Self {
        let mut row = Vec::with_capacity(20);
        for _ in 0..row.capacity() {
            row.push(' ');
        }
        let mut cols = Vec::with_capacity(15);
        for _ in 0..cols.capacity() {
            cols.push(row.clone());
        }
        DebugDisplay {
            inputs: inputs.into(),
            buffer: cols,
            selected_cursors: Vec::new(),
            unselected_cursors: Vec::new(),
        }
    }
}

impl Display for DebugDisplay {
    fn show(&mut self, state: &State) -> Result<(), ()> {
        draw(self, state, 15, 20)
    }
    fn getch(&mut self) -> Option<Input> {
        self.inputs.pop_front()
    }
}
impl DisplayDraw for DebugDisplay {
    fn erase(&mut self) -> Result<(), ()> {
        self.selected_cursors.clear();
        self.unselected_cursors.clear();
        for r in &mut self.buffer {
            for c in r {
                *c = ' ';
            }
        }
        Ok(())
    }
    fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), ()> {
        let c =
            match ch {
                Character::Character(ch) => ch,
                Character::VLine => '|',
                Character::HLine => '-',
            };
        if y > self.buffer.len() {
            Err(())
        } else {
            let row = &mut self.buffer[y];
            if x > row.len() {
                Err(())
            } else {
                row[x] = c;
                Ok(())
            }
        }
    }
    fn set_attribute(&mut self, y: usize, x: usize, at: Attribute) -> Result<(), ()> {
        match at {
            Attribute::SelectedCursor => self.selected_cursors.push((y, x)),
            Attribute::UnselectedCursor => self.unselected_cursors.push((y, x)),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout::Layout;
    use window::clone_window;

    #[test]
    fn debug_display_insertion() {
        let mut display = DebugDisplay::new(Vec::new());
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

        let state = State::new();
        display.show(&state).unwrap();
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        {
            let mut selected_window = state.selected_window.lock();
            let selected_window = &mut *selected_window;
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![]);
        assert_eq!(display.buffer,
                   vec!["abcd                ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_hsplit_insertion() {
        let mut display = DebugDisplay::new(Vec::new());
        let mut state = State::new();
        match state.layout {
            Layout::Window(window) => {
                let bottom = Box::new(Layout::Window(clone_window(&window)));
                state.layout = Layout::HSplit {
                    top: Box::new(Layout::Window(window)),
                    bottom,
                };
            },
            _ => unreachable!(),
        }
        display.show(&state).unwrap();
        assert_eq!(display.selected_cursors, vec![(0, 0)]);
        assert_eq!(display.unselected_cursors, vec![(8, 0)]);
        assert_eq!(display.buffer,
                   vec!["                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        {
            let mut selected_window = state.selected_window.lock();
            let selected_window = &mut *selected_window;
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![(8, 4)]);
        assert_eq!(display.buffer,
                   vec!["abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>(),
                        "--------------------".chars().collect::<Vec<_>>(),
                        "abcd                ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_vsplit_insertion() {
        let mut display = DebugDisplay::new(Vec::new());
        let mut state = State::new();
        match state.layout {
            Layout::Window(window) => {
                let right = Box::new(Layout::Window(clone_window(&window)));
                state.layout = Layout::VSplit {
                    left: Box::new(Layout::Window(window)),
                    right,
                };
            },
            _ => unreachable!(),
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["          |         ".chars().collect::<Vec<_>>(),
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
                        "*scratch* |*scratch*".chars().collect::<Vec<_>>()]);

        {
            let mut selected_window = state.selected_window.lock();
            let selected_window = &mut *selected_window;
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.selected_cursors, vec![(0, 4)]);
        assert_eq!(display.unselected_cursors, vec![(0, 15)]);
        assert_eq!(display.buffer,
                   vec!["abcd      |abcd     ".chars().collect::<Vec<_>>(),
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
                        "*scratch* |*scratch*".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_vsplit_deletion() {
        let mut display = DebugDisplay::new(Vec::new());
        let mut state = State::new();
        match state.layout {
            Layout::Window(window) => {
                state.layout = Layout::VSplit {
                    left: Box::new(Layout::Window(window.clone())),
                    right: Box::new(Layout::Window(window)),
                };
            },
            _ => unreachable!(),
        }
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        display.show(&state).unwrap();
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.delete_region(0, 2).unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["cd        |cd       ".chars().collect::<Vec<_>>(),
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
                        "*scratch* |*scratch*".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_multiple_lines() {
        let mut display = DebugDisplay::new(Vec::new());
        let state = State::new();
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abc\ndef").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abc                 ".chars().collect::<Vec<_>>(),
                        "def                 ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_wrap() {
        let mut display = DebugDisplay::new(Vec::new());
        let state = State::new();
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcdefghijklmnopqrstuvwxyz").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcdefghijklmnopqrst".chars().collect::<Vec<_>>(),
                        "uvwxyz              ".chars().collect::<Vec<_>>(),
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
                        "*scratch*           ".chars().collect::<Vec<_>>()]);

        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(26, "\nNow I know my ABCs.  Next time wont you sing with me.").unwrap();
            assert_eq!(format!("{}", *buffer), "abcdefghijklmnopqrstuvwxyz\nNow I know my ABCs.  Next time wont you sing with me.");
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["abcdefghijklmnopqrst".chars().collect::<Vec<_>>(),
                        "uvwxyz              ".chars().collect::<Vec<_>>(),
                        "Now I know my ABCs. ".chars().collect::<Vec<_>>(),
                        " Next time wont you ".chars().collect::<Vec<_>>(),
                        "sing with me.       ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "                    ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }

    #[test]
    fn debug_display_long_file() {
        let mut display = DebugDisplay::new(Vec::new());
        let state = State::new();
        {
            let selected_window = state.selected_window.lock();
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\nq\nr\ns\nt\nu\nv\nw\nx\ny\nz").unwrap();
        }
        display.show(&state).unwrap();
        assert_eq!(display.buffer,
                   vec!["a                   ".chars().collect::<Vec<_>>(),
                        "b                   ".chars().collect::<Vec<_>>(),
                        "c                   ".chars().collect::<Vec<_>>(),
                        "d                   ".chars().collect::<Vec<_>>(),
                        "e                   ".chars().collect::<Vec<_>>(),
                        "f                   ".chars().collect::<Vec<_>>(),
                        "g                   ".chars().collect::<Vec<_>>(),
                        "h                   ".chars().collect::<Vec<_>>(),
                        "i                   ".chars().collect::<Vec<_>>(),
                        "j                   ".chars().collect::<Vec<_>>(),
                        "k                   ".chars().collect::<Vec<_>>(),
                        "l                   ".chars().collect::<Vec<_>>(),
                        "m                   ".chars().collect::<Vec<_>>(),
                        "n                   ".chars().collect::<Vec<_>>(),
                        "*scratch*           ".chars().collect::<Vec<_>>()]);
    }
}
