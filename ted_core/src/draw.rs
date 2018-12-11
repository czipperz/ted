use layout::Layout;
use parking_lot::Mutex;
use renderer::Renderer;
use std::sync::Arc;
use window::Window;

pub enum Character {
    Character(char),
    VLine,
    HLine,
}

#[derive(PartialEq, Debug)]
pub enum Attribute {
    SelectedCursor,
    UnselectedCursor,
    Inverted,
}

pub trait DrawableRenderer: Renderer {
    fn erase(&mut self) -> Result<(), ()>;
    fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), ()>;
    fn set_attribute(&mut self, y: usize, x: usize, at: Attribute) -> Result<(), ()>;
}

pub fn draw<D>(
    display: &mut D,
    layout: &Layout,
    selected_window: &Arc<Mutex<Window>>,
    rows: usize,
    columns: usize,
) -> Result<(), ()>
where
    D: DrawableRenderer,
{
    layout.update_window_cursors();
    display.erase()?;
    draw_rect(display, layout, selected_window, 0, 0, rows, columns)?;
    Ok(())
}

fn draw_rect<D>(
    display: &mut D,
    layout: &Layout,
    selected_window: &Arc<Mutex<Window>>,
    y: usize,
    x: usize,
    rows: usize,
    columns: usize,
) -> Result<(), ()>
where
    D: DrawableRenderer,
{
    match layout {
        Layout::Window(window) => {
            let is_selected_window = Arc::ptr_eq(window, selected_window);
            let window = window.lock();
            let buffer = window.buffer.lock();
            let mut iter = buffer.iter();
            let mut column = 0;
            let mut row = 0;
            let mut location = 0;
            while row < rows - 1 {
                if window.cursor.get() == location {
                    if is_selected_window {
                        display.set_attribute(y + row, x + column, Attribute::SelectedCursor)?;
                    } else {
                        display.set_attribute(y + row, x + column, Attribute::UnselectedCursor)?;
                    }
                }
                match iter.next() {
                    Some('\n') => {
                        column = 0;
                        row += 1;
                    }
                    Some(ch) => {
                        display.putch(y + row, x + column, Character::Character(ch))?;
                        column += 1;
                        if column >= columns {
                            row += 1;
                            column = 0;
                        }
                    }
                    None => break,
                }
                location += 1;
            }
            let mut column = 0;
            for ch in buffer.name.display_name().chars() {
                if column < columns {
                    display.putch(y + rows - 1, x + column, Character::Character(ch))?;
                    column += 1;
                }
            }
            for column in 0..columns {
                display.set_attribute(y + rows - 1, x + column, Attribute::Inverted)?;
            }
            Ok(())
        }
        Layout::VSplit { left, right } => {
            // 4 columns
            // __|_
            // 5 columns
            // __|__
            draw_rect(display, left, selected_window, y, x, rows, columns / 2)?;
            for r in 0..rows {
                display.putch(y + r, x + columns / 2, Character::VLine)?;
            }
            draw_rect(
                display,
                right,
                selected_window,
                y,
                x + columns / 2 + 1,
                rows,
                (columns - 1) / 2,
            )?;
            Ok(())
        }
        Layout::HSplit { top, bottom } => {
            // 4 rows
            // __|_
            // 5 rows
            // __|__
            draw_rect(display, top, selected_window, y, x, rows / 2, columns)?;
            for c in 0..columns {
                display.putch(y + rows / 2, x + c, Character::HLine)?;
            }
            draw_rect(
                display,
                bottom,
                selected_window,
                y + rows / 2 + 1,
                x,
                (rows - 1) / 2,
                columns,
            )?;
            Ok(())
        }
    }
}
