use parking_lot::Mutex;
use std::sync::{Arc, Weak};

#[derive(Default)]
pub struct StateNode {
    pub pred: Weak<Mutex<StateNode>>,
    pub succ: Vec<Arc<Mutex<StateNode>>>,
    pub change: Change,
}

pub struct Change {
    pub loc: usize,
    pub s: String,
    pub len_chars: usize,
    pub is_insert: bool,
}

impl Change {
    pub fn offset_cursor_redo(&self, cursor: usize) -> usize {
        if self.is_insert {
            if cursor >= self.loc {
                cursor + self.len_chars
            } else {
                cursor
            }
        } else {
            if cursor > self.loc + self.len_chars {
                cursor - self.len_chars
            } else if cursor > self.loc {
                self.loc
            } else {
                cursor
            }
        }
    }

    pub fn offset_cursor_undo(&self, cursor: usize) -> usize {
        if !self.is_insert {
            if cursor >= self.loc {
                cursor + self.len_chars
            } else {
                cursor
            }
        } else {
            if cursor > self.loc + self.len_chars {
                cursor - self.len_chars
            } else if cursor > self.loc {
                self.loc
            } else {
                cursor
            }
        }
    }
}

impl Default for Change {
    fn default() -> Self {
        Change {
            loc: 0,
            s: String::default(),
            len_chars: 0,
            is_insert: true,
        }
    }
}
