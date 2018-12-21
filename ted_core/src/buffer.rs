use buffer_contents::*;
use by_address::ByAddress;
use change::*;
use mode::Mode;
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::path::*;
use std::sync::{Arc, Weak};

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
    _initial_state: Option<Arc<Mutex<StateNode>>>,
    current_state: Option<Arc<Mutex<StateNode>>>,
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
        Buffer {
            buffer_contents: contents,
            _initial_state: None,
            current_state: None,
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
        if self.read_only {
            return Err(
                "Error: Buffer::insert() cannot be called on a read only Buffer".to_string(),
            );
        }
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
        if self.read_only {
            return Err(
                "Error: Buffer::insert_str() cannot be called on a read only Buffer".to_string(),
            );
        }
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
        if self.read_only {
            return Err(
                "Error: Buffer::delete() cannot be called on a read only Buffer".to_string(),
            );
        }
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
        if self.read_only {
            return Err(
                "Error: Buffer::delete_region() cannot be called on a read only Buffer".to_string(),
            );
        }
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
        match self.current_state.take() {
            Some(current_state) => {
                let node = Arc::new(Mutex::new(StateNode {
                    pred: Arc::downgrade(&current_state),
                    succ: Vec::new(),
                    change,
                }));
                {
                    let mut current_state = current_state.lock();
                    current_state.succ.push(node.clone());
                }
                self.current_state = Some(node);
            }
            None => {
                let node = Arc::new(Mutex::new(StateNode {
                    pred: Weak::new(),
                    succ: Vec::new(),
                    change,
                }));
                self._initial_state = Some(node.clone());
                self.current_state = Some(node);
            }
        }
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
        if self.read_only {
            return Err("Error: Buffer::undo() cannot be called on a read only Buffer".to_string());
        }
        match self.current_state.take() {
            Some(current_state) => {
                let current_state = current_state.lock();
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
                self.current_state = current_state.pred.upgrade();
                Ok(true)
            }
            None => Ok(false),
        }
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
        if self.read_only {
            return Err("Error: Buffer::undo() cannot be called on a read only Buffer".to_string());
        }
        match self.current_state.take() {
            Some(current_state_lock) => {
                {
                    let current_state = current_state_lock.lock();
                    match current_state.succ.last() {
                        Some(next_state) => {
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
                            self.current_state = Some(next_state.clone());
                            return Ok(true);
                        }
                        None => {}
                    }
                }
                self.current_state = Some(current_state_lock);
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Erase the history of the `Buffer`.
    ///
    /// This function should probably only be called on `Buffer`s with
    /// [`BufferName::Internal`] names.
    ///
    /// [`BufferName::Internal`]: enum.BufferName.html#variant.Internal
    pub fn erase_history(&mut self) {
        self._initial_state = None;
        self.current_state = None;
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.buffer_contents)
    }
}

/// The name of the Buffer
pub struct BufferName {
    pub display_name: String,
    pub file_path: Option<PathBuf>,
}

impl<'a> From<&'a Path> for BufferName {
    fn from(p: &'a Path) -> Self {
        p.to_path_buf().into()
    }
}

impl From<PathBuf> for BufferName {
    fn from(p: PathBuf) -> Self {
        BufferName {
            display_name: p.file_name().unwrap().to_str().unwrap().to_string(),
            file_path: Some(p),
        }
    }
}

impl<'a> From<&'a str> for BufferName {
    fn from(s: &'a str) -> Self {
        s.to_string().into()
    }
}

impl From<String> for BufferName {
    fn from(display_name: String) -> Self {
        BufferName {
            display_name,
            file_path: None,
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
            *ret_state = match &buffer.current_state {
                Some(s) => {
                    *ret_location = s.lock().change.offset_cursor_redo(*ret_location);
                    Arc::downgrade(&s)
                }
                None => Weak::new(),
            };
            return;
        }
    };
    let mut location = *ret_location;
    if buffer.current_state.is_none() {
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
            if Arc::ptr_eq(&state, buffer.current_state.as_ref().unwrap()) {
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
    let state = state.upgrade();
    if buffer.current_state.is_none() && state.is_none() {
        true
    } else if buffer.current_state.is_none() || state.is_none() {
        false
    } else {
        let current_state = buffer.current_state.as_ref().unwrap();
        Arc::ptr_eq(current_state, &state.unwrap())
    }
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
}
