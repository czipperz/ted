use draw::*;
use input::Input;
use layout::Layout;
use parking_lot::Mutex;
use renderer::Renderer;
use std::collections::VecDeque;
use std::sync::Arc;
use window::Window;

/// An implementation of [`Renderer`] for debugging.
///
/// This renders a 20 column, 15 row renderer via a 2D array of
/// characters and allow for normal usage as a [`Renderer`].
///
/// You can emulate the user typing by pushing to `inputs`.
///
/// # Examples
///
/// ```
/// # use ted_core::{DebugRenderer, Renderer, State};
/// let renderer = DebugRenderer::new();
/// assert_eq!(renderer.buffer,
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
/// let state = State::new(renderer);
/// state.display.show().unwrap();
/// assert_eq!(unsafe { state.display.debug_renderer() }.buffer,
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
///     let buffer = state.display.selected_window_buffer();
///     let mut buffer = buffer.lock();
///     buffer.insert_str(0, "abcd").unwrap();
/// }
/// state.display.show().unwrap();
/// assert_eq!(unsafe { state.display.debug_renderer() }.selected_cursors, vec![(0, 4)]);
/// assert_eq!(unsafe { state.display.debug_renderer() }.unselected_cursors, vec![]);
/// assert_eq!(unsafe { state.display.debug_renderer() }.buffer,
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
/// [`Renderer`]: trait.Renderer.html
pub struct DebugRenderer {
    pub inputs: VecDeque<Input>,
    pub buffer: Vec<Vec<char>>,
    pub selected_cursors: Vec<(usize, usize)>,
    pub unselected_cursors: Vec<(usize, usize)>,
}

pub unsafe fn debug_renderer(renderer: &mut Renderer) -> &mut DebugRenderer {
    &mut *((renderer as *mut Renderer) as *mut DebugRenderer)
}

impl DebugRenderer {
    pub fn new() -> Self {
        DebugRenderer::from(Vec::new())
    }
}

impl From<Vec<Input>> for DebugRenderer {
    fn from(inputs: Vec<Input>) -> Self {
        let mut row = Vec::with_capacity(20);
        for _ in 0..row.capacity() {
            row.push(' ');
        }
        let mut cols = Vec::with_capacity(15);
        for _ in 0..cols.capacity() {
            cols.push(row.clone());
        }
        DebugRenderer {
            inputs: inputs.into(),
            buffer: cols,
            selected_cursors: Vec::new(),
            unselected_cursors: Vec::new(),
        }
    }
}

impl Renderer for DebugRenderer {
    fn show(&mut self, layout: &Layout, selected_window: &Arc<Mutex<Window>>) -> Result<(), ()> {
        draw(self, layout, selected_window, 15, 20)
    }
    fn getch(&mut self) -> Option<Input> {
        self.inputs.pop_front()
    }
}
impl DrawableRenderer for DebugRenderer {
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
        let c = match ch {
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
            Attribute::Inverted => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout::Layout;
    use state::State;
    use window::clone_window;

    fn debug_renderer(renderer: &Renderer) -> &DebugRenderer {
        unsafe { &*((&*renderer as *const Renderer) as *const DebugRenderer) }
    }

    #[test]
    fn debug_renderer_new() {
        let renderer = DebugRenderer::new();
        assert_eq!(
            renderer.buffer,
            vec![
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
                "                    ".chars().collect::<Vec<_>>(),
                "                    ".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_insertion() {
        let state = State::new(DebugRenderer::new());
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
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
                "                    ".chars().collect::<Vec<_>>(),
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );

        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.selected_cursors,
            vec![(0, 4)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.unselected_cursors,
            vec![]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abcd                ".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_hsplit_insertion() {
        let state = State::new(DebugRenderer::new());
        {
            let window = state
                .display
                .selected_frame
                .lock()
                .layout
                .unwrap_window()
                .clone();
            let bottom = Box::new(Layout::Window(clone_window(&window)));
            state.display.selected_frame.lock().layout = Layout::HSplit {
                top: Box::new(Layout::Window(window.clone())),
                bottom,
            };
        }
        state.display.show().unwrap();
        assert_eq!(
            debug_renderer(&*state.display.selected_frame.lock().renderer).selected_cursors,
            vec![(0, 0)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.unselected_cursors,
            vec![(8, 0)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "                    ".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );

        {
            let selected_window = state.display.selected_window();
            let mut selected_window = selected_window.lock();
            let selected_window = &mut *selected_window;
            let mut buffer = selected_window.buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.selected_cursors,
            vec![(0, 4)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.unselected_cursors,
            vec![(8, 4)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abcd                ".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_vsplit_insertion() {
        let state = State::new(DebugRenderer::new());
        {
            let window = state
                .display
                .selected_frame
                .lock()
                .layout
                .unwrap_window()
                .clone();
            let right = Box::new(Layout::Window(clone_window(&window)));
            state.display.selected_frame.lock().layout = Layout::VSplit {
                left: Box::new(Layout::Window(window.clone())),
                right,
            };
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
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
                "          |         ".chars().collect::<Vec<_>>(),
                "*scratch* |*scratch*".chars().collect::<Vec<_>>()
            ]
        );

        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.selected_cursors,
            vec![(0, 4)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.unselected_cursors,
            vec![(0, 15)]
        );
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abcd      |abcd     ".chars().collect::<Vec<_>>(),
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
                "*scratch* |*scratch*".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_vsplit_deletion() {
        let state = State::new(DebugRenderer::new());
        {
            let window = state
                .display
                .selected_frame
                .lock()
                .layout
                .unwrap_window()
                .clone();
            state.display.selected_frame.lock().layout = Layout::VSplit {
                left: Box::new(Layout::Window(window.clone())),
                right: Box::new(Layout::Window(window.clone())),
            };
        }
        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcd").unwrap();
        }
        state.display.show().unwrap();
        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.delete_region(0, 2).unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "cd        |cd       ".chars().collect::<Vec<_>>(),
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
                "*scratch* |*scratch*".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_multiple_lines() {
        let state = State::new(DebugRenderer::new());
        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abc\ndef").unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abc                 ".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_wrap() {
        let state = State::new(DebugRenderer::new());
        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer.insert_str(0, "abcdefghijklmnopqrstuvwxyz").unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abcdefghijklmnopqrst".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );

        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer
                .insert_str(
                    26,
                    "\nNow I know my ABCs.  Next time wont you sing with me.",
                ).unwrap();
            assert_eq!(
                format!("{}", *buffer),
                "abcdefghijklmnopqrstuvwxyz\nNow I know my ABCs.  Next time wont you sing with me."
            );
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "abcdefghijklmnopqrst".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );
    }

    #[test]
    fn debug_renderer_long_file() {
        let state = State::new(DebugRenderer::new());
        {
            let buffer = state.display.selected_window_buffer();
            let mut buffer = buffer.lock();
            buffer
                .insert_str(
                    0,
                    "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np\nq\nr\ns\nt\nu\nv\nw\nx\ny\nz",
                ).unwrap();
        }
        state.display.show().unwrap();
        assert_eq!(
            unsafe { state.display.debug_renderer() }.buffer,
            vec![
                "a                   ".chars().collect::<Vec<_>>(),
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
                "*scratch*           ".chars().collect::<Vec<_>>()
            ]
        );
    }
}
