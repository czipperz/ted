use buffer::*;
use cursor::Cursor;
use key_map::KeyMap;
use mode::Mode;
use parking_lot::Mutex;
use std::sync::Arc;

/// A view into a specific [`Buffer`]
///
/// [`Buffer`]: struct.Buffer.html
pub struct Window {
    pub buffer: Arc<Mutex<Buffer>>,
    pub buffer_key_map: Arc<Mutex<KeyMap>>,
    pub cursor: Cursor,
    pub window_modes: Vec<Arc<Mutex<Mode>>>,
}

impl Window {
    /// Create a window with a blank [`Buffer`].
    ///
    /// [`Buffer`]: struct.Buffer.html
    pub fn new() -> Self {
        Window::from(Buffer::new("*scratch*".into()))
    }

    /// Attempt to increment the cursor by `offset` chars.
    pub fn increment_cursor(&mut self, offset: isize) {
        let buffer = self.buffer.lock();
        self.cursor.increment(&*buffer, offset)
    }

    /// Attempt to set the cursor to `location`.
    pub fn set_cursor(&mut self, location: usize) {
        let buffer = self.buffer.lock();
        self.cursor.set(&*buffer, location)
    }

    /// Update the cursor to reflect new edits to the wrapped buffer.
    pub fn update_cursor(&mut self) {
        let buffer = self.buffer.lock();
        self.cursor.update(&*buffer);
    }

    /// Insert a char `c` at the cursor.
    ///
    /// This will automatically invoke [`update_cursor`].
    ///
    /// [`update_cursor`]: #method.update_cursor
    pub fn insert(&mut self, c: char) -> Result<(), ()> {
        let mut buffer = self.buffer.lock();
        buffer.insert(self.cursor.get(), c)?;
        self.cursor.update(&buffer);
        Ok(())
    }

    /// Insert a string `s` at the cursor.
    ///
    /// This will automatically invoke [`update_cursor`].
    ///
    /// [`update_cursor`]: #method.update_cursor
    pub fn insert_str(&mut self, s: &str) -> Result<(), ()> {
        let mut buffer = self.buffer.lock();
        buffer.insert_str(self.cursor.get(), s)?;
        self.cursor.update(&buffer);
        Ok(())
    }

    /// Delete 1 character at the cursor.
    ///
    /// This will automatically invoke [`update_cursor`].
    ///
    /// [`update_cursor`]: #method.update_cursor
    pub fn delete_1(&mut self) -> Result<(), ()> {
        let mut buffer = self.buffer.lock();
        buffer.delete(self.cursor.get())?;
        self.cursor.update(&buffer);
        Ok(())
    }

    /// Delete `n` chars at the cursor.
    ///
    /// This will automatically invoke [`update_cursor`].
    ///
    /// [`update_cursor`]: #method.update_cursor
    pub fn delete_n(&mut self, n: usize) -> Result<(), ()> {
        let end = self.cursor.get() + n;
        self.delete_until(end)
    }

    /// Delete the range from the cursor until `end`.
    ///
    /// This will automatically invoke [`update_cursor`].
    ///
    /// [`update_cursor`]: #method.update_cursor
    pub fn delete_until(&mut self, end: usize) -> Result<(), ()> {
        let mut buffer = self.buffer.lock();
        buffer.delete_region(self.cursor.get(), end)?;
        self.cursor.update(&buffer);
        Ok(())
    }
}

impl Default for Window {
    fn default() -> Self {
        Window::new()
    }
}

impl From<Buffer> for Window {
    fn from(buffer: Buffer) -> Self {
        Window {
            buffer: Arc::new(Mutex::new(buffer)),
            buffer_key_map: Arc::default(),
            cursor: Cursor::new(),
            window_modes: Vec::new(),
        }
    }
}

/// Make another window pointing to the same Buffer
///
/// This is needed because clone() couldn't handle the Buffer weak
/// pointing to the new Window (so cursor events are properly handled).
pub fn clone_window(window: &Arc<Mutex<Window>>) -> Arc<Mutex<Window>> {
    let window = window.lock();
    let cloned = Arc::new(Mutex::new(Window {
        buffer: window.buffer.clone(),
        buffer_key_map: window.buffer_key_map.clone(),
        cursor: window.cursor.clone(),
        window_modes: Vec::new(),
    }));
    cloned
}
