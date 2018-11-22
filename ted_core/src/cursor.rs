use std::sync::Weak;
use std::fmt;
use parking_lot::Mutex;
use change::StateNode;
use buffer::*;

#[derive(Clone)]
pub struct Cursor {
    location: usize,
    state: Weak<Mutex<StateNode>>,
}
impl Cursor {
    pub fn new() -> Self {
        Cursor { location: 0, state: Weak::new() }
    }

    pub fn get(&self) -> usize {
        self.location
    }

    pub fn increment(&mut self, buffer: &Buffer, offset: isize) {
        self.update(buffer);
        if offset < 0 && self.location < -offset as usize {
            self.location = 0;
        } else {
            let new_location = (self.location as isize + offset) as usize;
            if new_location > buffer.len() {
                self.location = buffer.len();
            } else {
                self.location = new_location;
            }
        }
    }

    pub fn set(&mut self, buffer: &Buffer, location: usize) {
        self.update(buffer);
        if location > buffer.len() {
            self.location = buffer.len();
        } else {
            self.location = location;
        }
    }

    pub fn unchecked_increment(&mut self, offset: isize) {
        self.location = (self.location as isize + offset) as usize;
    }

    pub fn unchecked_set(&mut self, location: usize) {
        self.location = location;
    }

    pub fn update(&mut self, buffer: &Buffer) {
        update_cursor(buffer, &mut self.state, &mut self.location);
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location &&
            match (self.state.upgrade(), other.state.upgrade()) {
                (Some(s), Some(o)) => std::sync::Arc::ptr_eq(&s, &o),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Eq for Cursor {}

impl fmt::Debug for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.location)
    }
}

#[cfg(test)]
mod tests {
    use window::Window;

    #[test]
    fn window_increment_cursor() {
        let mut window = Window::new();
        {
            let mut buffer = window.buffer.lock();
            buffer.insert_str(0, "abc").unwrap();
            window.cursor.set(&buffer, 0);
        }
        assert_eq!(window.cursor.get(), 0);
        window.increment_cursor(1);
        assert_eq!(window.cursor.get(), 1);
        window.increment_cursor(1);
        assert_eq!(window.cursor.get(), 2);
        window.increment_cursor(-1);
        assert_eq!(window.cursor.get(), 1);
        window.increment_cursor(2);
        assert_eq!(window.cursor.get(), 3);
        window.increment_cursor(-4);
        assert_eq!(window.cursor.get(), 0);
    }

    #[test]
    fn window_set_cursor() {
        let mut window = Window::new();
        {
            let mut buffer = window.buffer.lock();
            buffer.insert_str(0, "abc").unwrap();
            window.cursor.set(&buffer, 0);
        }
        assert_eq!(window.cursor.get(), 0);
        window.set_cursor(0);
        assert_eq!(window.cursor.get(), 0);
        window.set_cursor(2);
        assert_eq!(window.cursor.get(), 2);
        window.set_cursor(3);
        assert_eq!(window.cursor.get(), 3);
        window.set_cursor(1);
        assert_eq!(window.cursor.get(), 1);
        window.set_cursor(100);
        assert_eq!(window.cursor.get(), 3);
    }
}
