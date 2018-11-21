use std::fmt;
use std::sync::{Arc, Weak};
use parking_lot::Mutex;
use by_address::ByAddress;
use buffer_contents::*;
use change::*;
use std::collections::{HashSet, VecDeque};

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
        Buffer {
            buffer_contents: BufferContents::new(),
            _initial_state: None,
            current_state: None,
            name,
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
    pub fn len(&self) -> usize { self.buffer_contents.len() }

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
    pub fn iter(&self) -> BufferContentsIterator { self.buffer_contents.iter() }

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
    pub fn substring(&self, begin: usize, end: usize) -> Result<String, ()> {
        self.buffer_contents.substring(begin, end)
    }

    /// Insert char `c` at point `loc`.
    pub fn insert(&mut self, loc: usize, c: char) -> Result<(), ()> {
        self.buffer_contents.insert(loc, c)?;
        self.add_change(Change { loc, s: c.to_string(), len_chars: 1, is_insert: true });
        Ok(())
    }

    /// Insert string `s` at point `loc`.
    pub fn insert_str(&mut self, loc: usize, s: &str) -> Result<(), ()> {
        self.buffer_contents.insert_str(loc, s)?;
        self.add_change(Change { loc, s: s.to_string(),
                                 len_chars: s.chars().count(), is_insert: true });
        Ok(())
    }

    /// Delete the character at point `loc`.
    pub fn delete(&mut self, loc: usize) -> Result<(), ()> {
        let c = self.get(loc)?;
        self.buffer_contents.delete(loc)?;
        self.add_change(Change { loc, s: c.to_string(), len_chars: 1, is_insert: false });
        Ok(())
    }

    /// Delete the region from position `begin` up until `end`.
    pub fn delete_region(&mut self, begin: usize, end: usize) -> Result<(), ()> {
        let s = self.substring(begin, end)?;
        self.buffer_contents.delete_region(begin, end)?;
        let len_chars = s.chars().count();
        self.add_change(Change { loc: begin, s: s,
                                 len_chars, is_insert: false });
        Ok(())
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
            },
            None => {
                let node = Arc::new(Mutex::new(StateNode {
                    pred: Weak::new(),
                    succ: Vec::new(),
                    change,
                }));
                self._initial_state = Some(node.clone());
                self.current_state = Some(node);
            },
        }
    }

    /// Undo the last change.
    ///
    /// This will revert the last change made to the buffer.  Any call
    /// to `insert`, `insert_str`, `delete`, or `delete_range` is
    /// considered a change.
    ///
    /// If there are no edits to undo, 
    ///
    /// ## Examples
    ///
    /// ```
    /// # use ted_core::{Buffer, BufferName};
    /// let mut buffer = Buffer::new("*scratch*".into());
    /// buffer.insert_str(0, "ac").unwrap();
    /// buffer.insert(1, 'b').unwrap();
    /// assert_eq!(format!("{}", buffer), "abc");
    ///
    /// assert!(buffer.undo());
    /// assert_eq!(format!("{}", buffer), "ac");
    ///
    /// assert!(buffer.undo());
    /// assert_eq!(format!("{}", buffer), "");
    ///
    /// assert!(!buffer.undo());
    /// assert_eq!(format!("{}", buffer), "");
    /// ```
    pub fn undo(&mut self) -> bool {
        match self.current_state.take() {
            Some(current_state) => {
                let current_state = current_state.lock();
                if current_state.change.is_insert {
                    self.buffer_contents.delete_region
                        (current_state.change.loc,
                         current_state.change.loc + current_state.change.len_chars)
                        .unwrap();
                } else {
                    self.buffer_contents.insert_str(current_state.change.loc,
                                                    &current_state.change.s)
                        .unwrap();
                }
                self.current_state = current_state.pred.upgrade();
                true
            },
            None => {
                false
            },
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
    /// assert!(!buffer.redo());
    /// assert_eq!(format!("{}", buffer), "abc");
    ///
    /// assert!(buffer.undo());
    /// assert_eq!(format!("{}", buffer), "ac");
    ///
    /// assert!(buffer.redo());
    /// assert_eq!(format!("{}", buffer), "abc");
    /// ```
    pub fn redo(&mut self) -> bool {
        match self.current_state.take() {
            Some(current_state_lock) => {
                {
                    let current_state = current_state_lock.lock();
                    match current_state.succ.last() {
                        Some(next_state) => {
                            {
                                let next_state = next_state.lock();
                                if !next_state.change.is_insert {
                                    self.buffer_contents.delete_region
                                        (next_state.change.loc,
                                         next_state.change.loc + next_state.change.len_chars)
                                        .unwrap();
                                } else {
                                    self.buffer_contents.insert_str(next_state.change.loc,
                                                                    &next_state.change.s)
                                        .unwrap();
                                }
                            }
                            self.current_state = Some(next_state.clone());
                            return true;
                        },
                        None => {},
                    }
                }
                self.current_state = Some(current_state_lock);
                false
            },
            None => {
                false
            },
        }
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.buffer_contents)
    }
}

/// The name of the Buffer
pub enum BufferName {
    File { name: String, path: String },
    Internal(String),
}

impl<T: Into<String>> From<T> for BufferName {
    fn from(t: T) -> Self {
        BufferName::Internal(t.into())
    }
}

pub fn update_cursor(buffer: &Buffer, ret_state: &mut Weak<Mutex<StateNode>>, ret_location: &mut usize) {
    let mut state =
        match ret_state.upgrade() {
            Some(state) => state,
            None => {
                debug_assert_eq!(*ret_location, 0);
                *ret_state = match &buffer.current_state {
                    Some(s) => Arc::downgrade(&s),
                    None => Weak::new(),
                };
                *ret_location = buffer.len();
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
                },
                None => {},
            }
        }
        let new_state = states.pop_front().unwrap();
        state = new_state.0;
        location = new_state.1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_1() {
        let mut buffer = Buffer::new("*scratch*".into());
        assert_eq!(format!("{}", buffer), "");
        assert!(!buffer.undo());
        assert_eq!(format!("{}", buffer), "");
        buffer.insert_str(0, "Helllo World").unwrap();
        assert_eq!(format!("{}", buffer), "Helllo World");
        buffer.delete(2).unwrap();
        assert_eq!(format!("{}", buffer), "Hello World");
        assert!(buffer.undo());
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert!(buffer.undo());
        assert_eq!(format!("{}", buffer), "");
        assert!(!buffer.undo());
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

        assert!(buffer.undo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);

        assert!(buffer.redo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hello World");
        assert_eq!(cursor.get(), 11);

        assert!(!buffer.redo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Hello World");
        assert_eq!(cursor.get(), 11);

        assert!(buffer.undo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "Helllo World");
        assert_eq!(cursor.get(), 12);

        assert!(buffer.undo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        assert!(!buffer.undo());
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

        assert!(buffer.undo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);

        assert!(!buffer.undo());
        cursor.update(&buffer);
        assert_eq!(format!("{}", buffer), "");
        assert_eq!(cursor.get(), 0);
    }
}
