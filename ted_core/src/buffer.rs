use buffer_contents::*;
use by_address::ByAddress;
use change::*;
use mode::Mode;
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::path::*;
use std::sync::{Arc, Weak};

const ERROR_READ_ONLY: &'static str = "Error: Buffer is read only";

/// The actual text storage structure
///
/// This stores both the contents of the `Buffer` and the undo tree.
/// All locations are stored by character position.
///
/// # Examples
///
/// Basic insertion:
/// ```
/// # use ted_core::{Buffer, BufferName};
/// let mut buffer = Buffer::new("*scratch*".into());
/// buffer.insert_str(0, "αγ").unwrap();
/// buffer.insert_str(1, "βθ").unwrap();
/// assert_eq!(buffer.len(), 4);
/// assert_eq!(format!("{}", buffer), "αβθγ");
/// ```
///
/// Basic deletion:
/// ```
/// # use ted_core::{Buffer, BufferName};
/// let mut buffer = Buffer::new("*scratch*".into());
/// buffer.insert_str(0, "abcd");
/// buffer.delete_region(2, 4).unwrap();
/// assert_eq!(buffer.len(), 2);
/// assert_eq!(format!("{}", buffer), "ab");
/// ```
pub struct Buffer {
    buffer_contents: BufferContents,
    initial_state: Arc<Mutex<StateNode>>,
    current_state: Arc<Mutex<StateNode>>,
    /// The name of the buffer.
    pub name: BufferName,
    pub buffer_modes: Vec<Arc<Mutex<Mode>>>,
    pub read_only: bool,
}

impl Buffer {
    /// Create a blank `Buffer`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// assert_eq!(buffer.len(), 0);
    /// ```
    pub fn new(name: BufferName) -> Self {
        Buffer::new_with_buffer_contents(name, BufferContents::new())
    }

    /// Create a `Buffer` with some inital contents.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new_with_contents("*scratch*".into(), "abc");
    /// assert_eq!(buffer.len(), 3);
    /// assert_eq!(buffer.to_string(), "abc");
    /// ```
    pub fn new_with_contents(name: BufferName, contents: &str) -> Self {
        Buffer::new_with_buffer_contents(name, contents.into())
    }

    fn new_with_buffer_contents(name: BufferName, contents: BufferContents) -> Self {
        let state: Arc<Mutex<StateNode>> = Arc::default();
        Buffer {
            buffer_contents: contents,
            initial_state: state.clone(),
            current_state: state,
            name,
            buffer_modes: Vec::new(),
            read_only: false,
        }
    }

    /// Retrieve the number of characters in the `Buffer`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "αβθγ");
    /// assert_eq!(buffer.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.buffer_contents.len()
    }

    /// Iterate over the contents of the `Buffer`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "αβθγ");
    /// let mut iter = buffer.iter();
    /// assert_eq!(iter.next(), Some('α'));
    /// assert_eq!(iter.next(), Some('β'));
    /// assert_eq!(iter.next(), Some('θ'));
    /// assert_eq!(iter.next(), Some('γ'));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> BufferContentsIterator {
        self.buffer_contents.iter()
    }

    /// Get the character at position `loc`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "αβθγ");
    /// assert_eq!(buffer.get(0).unwrap(), 'α');
    /// assert_eq!(buffer.get(1).unwrap(), 'β');
    /// assert_eq!(buffer.get(2).unwrap(), 'θ');
    /// assert_eq!(buffer.get(3).unwrap(), 'γ');
    /// assert!(buffer.get(4).is_err());
    /// ```
    pub fn get(&self, loc: usize) -> Result<char, ()> {
        self.buffer_contents.get(loc)
    }

    /// Retrieve a substring from position `begin` up until `end`.
    pub fn substring(&self, begin: usize, end: usize) -> Result<String, String> {
        self.buffer_contents
            .substring(begin, end)
            .map_err(|()| "Error: Index out of bounds in Buffer::substring()".to_string())
    }

    /// Insert char `c` at point `loc`.
    pub fn insert(&mut self, loc: usize, c: char) -> Result<(), String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        self.buffer_contents
            .insert(loc, c)
            .map_err(|()| "Error: Index out of bounds in Buffer::insert()")?;
        self.add_change(Change {
            loc,
            s: c.to_string(),
            len_chars: 1,
            is_insert: true,
        });
        Ok(())
    }

    /// Insert string `s` at point `loc`.
    pub fn insert_str(&mut self, loc: usize, s: &str) -> Result<(), String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        self.buffer_contents
            .insert_str(loc, s)
            .map_err(|()| "Error: Index out of bounds in Buffer::insert_str()".to_string())?;
        self.add_change(Change {
            loc,
            s: s.to_string(),
            len_chars: s.chars().count(),
            is_insert: true,
        });
        Ok(())
    }

    /// Delete the character at point `loc`.
    pub fn delete(&mut self, loc: usize) -> Result<(), String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        let c = self
            .get(loc)
            .map_err(|()| "Error: Index out of bounds in Buffer::delete()".to_string())?;
        self.buffer_contents
            .delete(loc)
            .map_err(|()| "Error: Index out of bounds in Buffer::delete()".to_string())?;
        self.add_change(Change {
            loc,
            s: c.to_string(),
            len_chars: 1,
            is_insert: false,
        });
        Ok(())
    }

    /// Delete the region from position `begin` up until `end`.
    pub fn delete_region(&mut self, begin: usize, end: usize) -> Result<(), String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        let s = self.substring(begin, end)?;
        self.buffer_contents
            .delete_region(begin, end)
            .map_err(|()| "Error: Index out of bounds in Buffer::delete_region()".to_string())?;
        let len_chars = s.chars().count();
        self.add_change(Change {
            loc: begin,
            s: s,
            len_chars,
            is_insert: false,
        });
        Ok(())
    }

    /// Clear the `Buffer`, deleting the entire thing.
    pub fn clear(&mut self) -> Result<(), String> {
        let len = self.len();
        self.delete_region(0, len)
    }

    /// Handle adding another node to the state graph and pointing `current_state` to it.
    fn add_change(&mut self, change: Change) {
        let node = Arc::new(Mutex::new(StateNode {
            pred: Arc::downgrade(&self.current_state),
            succ: Vec::new(),
            change,
        }));
        {
            let mut current_state = self.current_state.lock();
            current_state.succ.push(node.clone());
        }
        self.current_state = node;
    }

    /// Undo the last change.
    ///
    /// This will revert the last change made to the buffer.  Any call
    /// to `insert`, `insert_str`, `delete`, or `delete_range` is
    /// considered a change.
    ///
    /// If there are no edits to undo, returns false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "ac").unwrap();
    /// buffer.insert(1, 'b').unwrap();
    /// assert_eq!(format!("{}", buffer), "abc");
    ///
    /// assert!(buffer.undo().unwrap());
    /// assert_eq!(format!("{}", buffer), "ac");
    ///
    /// assert!(buffer.undo().unwrap());
    /// assert_eq!(format!("{}", buffer), "");
    ///
    /// assert!(!buffer.undo().unwrap());
    /// assert_eq!(format!("{}", buffer), "");
    /// ```
    pub fn undo(&mut self) -> Result<bool, String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        let pred;
        {
            let current_state = self.current_state.lock();
            pred = match current_state.pred.upgrade() {
                Some(pred) => pred,
                None => return Ok(false),
            };
            if current_state.change.is_insert {
                self.buffer_contents
                    .delete_region(
                        current_state.change.loc,
                        current_state.change.loc + current_state.change.len_chars,
                    ).map_err(|()| "Error: Index out of bounds in Buffer::undo()".to_string())?;
            } else {
                self.buffer_contents
                    .insert_str(current_state.change.loc, &current_state.change.s)
                    .map_err(|()| "Error: Index out of bounds in Buffer::undo()".to_string())?;
            }
        }
        self.current_state = pred;
        Ok(true)
    }

    /// Redo the last change if it has been undone.
    ///
    /// This method specifically does not cause the last change to be
    /// done twice, but reverts an `undo`.
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "ac").unwrap();
    /// buffer.insert(1, 'b').unwrap();
    /// assert_eq!(format!("{}", buffer), "abc");
    ///
    /// // Redo has nothing to redo
    /// assert!(!buffer.redo().unwrap());
    /// assert_eq!(format!("{}", buffer), "abc");
    ///
    /// assert!(buffer.undo().unwrap());
    /// assert_eq!(format!("{}", buffer), "ac");
    ///
    /// assert!(buffer.redo().unwrap());
    /// assert_eq!(format!("{}", buffer), "abc");
    /// ```
    pub fn redo(&mut self) -> Result<bool, String> {
        if self.read_only { Err(ERROR_READ_ONLY)? }
        let current_state = self.current_state.clone();
        let current_state = current_state.lock();
        if let Some(next_state) = current_state.succ.last() {
            {
                let next_state = next_state.lock();
                if !next_state.change.is_insert {
                    self.buffer_contents
                        .delete_region(
                            next_state.change.loc,
                            next_state.change.loc + next_state.change.len_chars,
                        ).map_err(|()| {
                            "Error: Index out of bounds in Buffer::redo()"
                                .to_string()
                        })?;
                } else {
                    self.buffer_contents
                        .insert_str(next_state.change.loc, &next_state.change.s)
                        .map_err(|()| {
                            "Error: Index out of bounds in Buffer::redo()"
                                .to_string()
                        })?;
                }
            }
            self.current_state = next_state.clone();
            return Ok(true);
        }
        Ok(false)
    }

    /// Erase the history of the `Buffer`.
    pub fn erase_history(&mut self) {
        let state: Arc<Mutex<StateNode>> = Arc::default();
        self.initial_state = state.clone();
        self.current_state = state;
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.buffer_contents)
    }
}

/// The name of the Buffer
pub struct BufferName {
    pub name: String,
    pub path: Option<PathBuf>,
}

impl BufferName {
    pub fn parent(&self) -> Option<&Path> {
        self.path.as_ref().and_then(|p| p.parent())
    }
}

impl<'a> From<&'a Path> for BufferName {
    fn from(p: &'a Path) -> Self {
        p.to_path_buf().into()
    }
}

impl From<PathBuf> for BufferName {
    fn from(p: PathBuf) -> Self {
        BufferName {
            name: p.file_name().unwrap().to_str().unwrap().to_string(),
            path: Some(p),
        }
    }
}

impl<'a> From<&'a str> for BufferName {
    fn from(s: &'a str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for BufferName {
    fn from(name: String) -> Self {
        BufferName {
            name,
            path: None,
        }
    }
}

pub fn update_cursor(
    buffer: &Buffer,
    ret_state: &mut Weak<Mutex<StateNode>>,
    ret_location: &mut usize,
) {
    let mut state = match ret_state.upgrade() {
        Some(state) => state,
        None => {
            if !Arc::ptr_eq(&buffer.current_state, &buffer.initial_state) {
                *ret_location = buffer.current_state.lock().change.offset_cursor_redo(*ret_location);
                *ret_state = Arc::downgrade(&buffer.current_state)
            }
            return;
        }
    };
    let mut location = *ret_location;
    if Arc::ptr_eq(&buffer.current_state, &buffer.initial_state) {
        let mut state_lock = state;
        loop {
            state_lock = {
                let state = state_lock.lock();
                location = state.change.offset_cursor_undo(location);
                match state.pred.upgrade() {
                    Some(pred) => pred,
                    None => break,
                }
            }
        }
        *ret_state = Weak::new();
        *ret_location = location;
        return;
    }
    let mut history = HashSet::new();
    let mut states = VecDeque::new();
    loop {
        if history.insert(ByAddress(state.clone())) {
            // haven't seen this item before
            if Arc::ptr_eq(&state, &buffer.current_state) {
                *ret_state = Arc::downgrade(&state);
                *ret_location = location;
                return;
            }

            let state = state.lock();
            // handle successors
            for new_state in &state.succ {
                let new_location = {
                    let new_state = new_state.lock();
                    new_state.change.offset_cursor_redo(location)
                };
                states.push_back((new_state.clone(), new_location));
            }
            // handle predecessors
            match state.pred.upgrade() {
                Some(new_state) => {
                    let new_location = state.change.offset_cursor_undo(location);
                    states.push_back((new_state.clone(), new_location));
                }
                None => {}
            }
        }
        let new_state = states.pop_front().unwrap();
        state = new_state.0;
        location = new_state.1;
    }
}

pub fn is_updated_cursor(buffer: &Buffer, state: &Weak<Mutex<StateNode>>) -> bool {
    Arc::ptr_eq(&buffer.current_state, &state.upgrade().as_ref().unwrap_or(&buffer.initial_state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_1() {
        let mut buffer = Buffer::new("*scratch*".into());
        assert_eq!(format!("{}", buffer), "");
        assert!(!buffer.undo().unwrap());
        assert_eq!(format!("{}", buffer), "");
        buffer.insert_str(0, "Helllo World").unwrap();
        assert_eq!(format!("{}", buffer), "Helllo World");
        buffer.delete(2).unwrap();
        assert_eq!(format!("{}", buffer), "Hello World");
        assert!(buffer.undo().unwrap());
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert!(buffer.undo().unwrap());
        assert_eq!(format!("{}", buffer), "");
        assert!(!buffer.undo().unwrap());
        assert_eq!(format!("{}", buffer), "");
    }

    #[test]
    fn undo_redo_1() {
        use cursor::Cursor;
        let mut buffer = Buffer::new("*scratch*".into());
        let mut cursor = Cursor::new();

        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        buffer.insert_str(0, "Helllo World").unwrap();
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);

        buffer.delete(2).unwrap();
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hello World");
        assert_eq!(cursor.get(), 11);

        assert!(buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);

        assert!(buffer.redo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hello World");
        assert_eq!(cursor.get(), 11);

        assert!(!buffer.redo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hello World");
        assert_eq!(cursor.get(), 11);

        assert!(buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);

        assert!(buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        assert!(!buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        assert!(buffer.redo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);
    }

    #[test]
    fn undo_already_undone_all_changes() {
        use cursor::Cursor;
        let mut buffer = Buffer::new("*scratch*".into());
        let mut cursor = Cursor::new();

        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        buffer.insert_str(0, "Hi").unwrap();
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hi");
        assert_eq!(cursor.get(), 2);

        assert!(buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        assert!(!buffer.undo().unwrap());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);
    }

    #[test]
    fn new_with_contents_redo_after_undo() {
        let mut buffer = Buffer::new_with_contents("*scratch*".into(), "Example text");

        assert_eq!(buffer.to_string(), "Example text");
        buffer.insert_str(0, "Some ").unwrap();
        assert_eq!(buffer.to_string(), "Some Example text");
        buffer.undo().unwrap();
        assert_eq!(buffer.to_string(), "Example text");
        buffer.redo().unwrap();
        assert_eq!(buffer.to_string(), "Some Example text");
    }
}
