use buffer::*;
use change::StateNode;
use parking_lot::Mutex;
use std::fmt;
use std::sync::Weak;

/// Safely index into a [`Buffer`] and get automatic updates.
///
/// In general terms, the `Cursor` stores a position and the
/// associated state.  When the [`Buffer`] is updated, neither of
/// those fields will be updated as the [`Cursor`] will be stuck on
/// the old state.  Thus it is critical to invoke [`update`] when
/// making changes to the [`Buffer`].
///
/// Many convenience methods for Cursor usage are located in
/// [`Window`].  In addition the convenience methods for interacting
/// with the [`Buffer`] located in [`Window`] will properly update the
/// `Cursor`.
///
/// [`update`]: #method.update
/// [`Buffer`]: struct.Buffer.html
/// [`Window`]: struct.Window.html
#[derive(Clone)]
pub struct Cursor {
    location: usize,
    state: Weak<Mutex<StateNode>>,
}
impl Cursor {
    /// Create a `Cursor` at location 0 and a null state.
    pub fn new() -> Self {
        Cursor {
            location: 0,
            state: Weak::new(),
        }
    }

    /// Get the `Cursor`'s location
    pub fn get(&self) -> usize {
        self.location
    }

    /// Increment the `Cursor`'s location.
    ///
    /// This will keep the `Cursor` inside the boundary of the [`Buffer`].
    ///
    /// This will automatically update the `Cursor` to reflect the
    /// [`Buffer`]'s state before incrementing the `Cursor`.
    ///
    /// [`Buffer`]: struct.Buffer.html
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

    /// Set the `Cursor`'s location.
    ///
    /// This will keep the `Cursor` inside the boundary of the [`Buffer`].
    ///
    /// This will automatically update the `Cursor` to reflect the
    /// [`Buffer`]'s state before incrementing the `Cursor`.  The
    /// `location` is assumed to be indexing into this current state.
    ///
    /// [`Buffer`]: struct.Buffer.html
    pub fn set(&mut self, buffer: &Buffer, location: usize) {
        self.update(buffer);
        if location > buffer.len() {
            self.location = buffer.len();
        } else {
            self.location = location;
        }
    }

    /// Update the `Cursor` to the current state of the [`Buffer`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, Cursor};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// let mut cursor = Cursor::new();
    ///
    /// buffer.insert_str(0, "abef").unwrap();
    /// cursor.set(&buffer, 2);
    /// assert_eq!(cursor.get(), 2);
    ///
    /// buffer.insert_str(2, "cd").unwrap();
    /// assert_eq!(cursor.get(), 2);
    /// cursor.update(&buffer);
    /// assert_eq!(cursor.get(), 4);
    /// ```
    pub fn update(&mut self, buffer: &Buffer) {
        update_cursor(buffer, &mut self.state, &mut self.location);
    }

    /// Update the `Cursor` to the current state of the [`Buffer`].
    ///
    /// This allows for easier access of a [`Window`]'s updated cursor
    /// while the [`Window`] is still immutable.
    ///
    /// [`Window`]: struct.Window.html
    pub fn updated(mut self, buffer: &Buffer) -> Self {
        self.update(buffer);
        self
    }

    /// Indicates whether the `Cursor` has been updated properly.
    pub fn is_updated(&self, buffer: &Buffer) -> bool {
        is_updated_cursor(buffer, &self.state)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::new()
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location && match (self.state.upgrade(), other.state.upgrade()) {
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
    use super::*;

    #[test]
    fn buffer_with_contents_cursor_movement_update() {
        let buffer = Buffer::new_with_contents("*scratch*".into(), "contents\n");
        let mut cursor = Cursor::new();
        assert_eq!(cursor.get(), 0);
        cursor.update(&buffer);
        assert_eq!(cursor.get(), 9);
        cursor.increment(&buffer, -3);
        assert_eq!(cursor.get(), 6);
    }

    #[test]
    fn buffer_with_contents_cursor_movement_noupdate() {
        let buffer = Buffer::new_with_contents("*scratch*".into(), "contents\n");
        let mut cursor = Cursor::new();
        assert_eq!(cursor.get(), 0);
        cursor.increment(&buffer, -1);
        assert_eq!(cursor.get(), 8);
    }
}
