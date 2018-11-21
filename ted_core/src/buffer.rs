use std::fmt;
use std::sync::{Arc, Weak};
use parking_lot::Mutex;
use by_address::ByAddress;
use buffer_contents::*;
use change::*;
use std::collections::{HashSet, VecDeque};

pub struct Buffer {
    buffer_contents: BufferContents,
    _initial_state: Option<Arc<Mutex<StateNode>>>,
    current_state: Option<Arc<Mutex<StateNode>>>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer_contents: BufferContents::new(),
            _initial_state: None,
            current_state: None,
        }
    }

    pub fn len(&self) -> usize { self.buffer_contents.len() }

    pub fn iter(&self) -> BufferContentsIterator { self.buffer_contents.iter() }

    pub fn get(&self, loc: usize) -> Result<char, ()> {
        self.buffer_contents.get(loc)
    }

    pub fn substring(&self, begin: usize, end: usize) -> Result<String, ()> {
        self.buffer_contents.substring(begin, end)
    }

    pub fn insert(&mut self, loc: usize, c: char) -> Result<(), ()> {
        self.buffer_contents.insert(loc, c)?;
        self.add_change(Change { loc, s: c.to_string(), len_chars: 1, is_insert: true });
        Ok(())
    }

    pub fn insert_str(&mut self, loc: usize, s: &str) -> Result<(), ()> {
        self.buffer_contents.insert_str(loc, s)?;
        self.add_change(Change { loc, s: s.to_string(),
                                 len_chars: s.chars().count(), is_insert: true });
        Ok(())
    }

    pub fn delete(&mut self, loc: usize) -> Result<(), ()> {
        let c = self.get(loc)?;
        self.buffer_contents.delete(loc)?;
        self.add_change(Change { loc, s: c.to_string(), len_chars: 1, is_insert: false });
        Ok(())
    }

    pub fn delete_region(&mut self, begin: usize, end: usize) -> Result<(), ()> {
        let s = self.substring(begin, end)?;
        self.buffer_contents.delete_region(begin, end)?;
        let len_chars = s.chars().count();
        self.add_change(Change { loc: begin, s: s,
                                 len_chars, is_insert: false });
        Ok(())
    }

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

impl<'a> From<&'a str> for Buffer {
    fn from(s: &str) -> Self {
        Buffer {
            buffer_contents: BufferContents::from(s),
            _initial_state: None,
            current_state: None,
        }
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
        let mut buffer = Buffer::new();
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
        let mut buffer = Buffer::new();
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
        let mut buffer = Buffer::new();
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
