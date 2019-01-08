/// A user input event
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input {
    /// Was control held
    pub control: bool,
    /// Was alt held
    pub alt: bool,
    /// The key
    pub key: Key,
}

/// A key value
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Key(char),
    Function(i8),
}

impl Input {
    pub fn is_unmodified(&self) -> bool {
        !self.control && !self.alt
    }
}

pub const BACKSPACE: Key = Key::Key(127 as char);

use std::fmt;
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.control {
            write!(f, "C-")?;
        }
        if self.alt {
            write!(f, "A-")?;
        }
        write!(f, "{:?}", self.key)
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Key::Key(c) => write!(f, "{}", c),
            &Key::Function(function) => write!(f, "F{}", function),
        }
    }
}

/// This function aids in creation of key bindings.
///
/// # Examples
///
/// ```
/// # use ted_core::{kbd, Key, Input, BACKSPACE};
/// assert_eq!(kbd("a"), Input { key: Key::Key('a'), control: false, alt: false });
/// assert_eq!(kbd("C-a"), Input { key: Key::Key('a'), control: true, alt: false });
/// assert_eq!(kbd("A-a"), Input { key: Key::Key('a'), control: false, alt: true });
/// assert_eq!(kbd("C-A-a"), Input { key: Key::Key('a'), control: true, alt: true });
/// assert_eq!(kbd("F1"), Input { key: Key::Function(1), control: false, alt: false });
/// assert_eq!(kbd("C-F1"), Input { key: Key::Function(1), control: true, alt: false });
/// assert_eq!(kbd("A-F1"), Input { key: Key::Function(1), control: false, alt: true });
/// assert_eq!(kbd("C-A-F1"), Input { key: Key::Function(1), control: true, alt: true });
/// assert_eq!(kbd("Backspace"), Input { key: BACKSPACE, control: false, alt: false });
/// assert_eq!(kbd("C-Backspace"), Input { key: BACKSPACE, control: true, alt: false });
/// assert_eq!(kbd("A-Backspace"), Input { key: BACKSPACE, control: false, alt: true });
/// assert_eq!(kbd("C-A-Backspace"), Input { key: BACKSPACE, control: true, alt: true });
/// ```
pub fn kbd(k: &str) -> Input {
    if k.starts_with("C-") {
        let mut i = kbd(&k[2..]);
        i.control = true;
        i
    } else if k.starts_with("A-") {
        let mut i = kbd(&k[2..]);
        i.alt = true;
        i
    } else if k == "Backspace" {
        Input {
            control: false,
            alt: false,
            key: BACKSPACE,
        }
    } else if k.starts_with('F') {
        Input {
            control: false,
            alt: false,
            key: Key::Function(
                k[1..]
                    .parse()
                    .expect("In kbd(): 'F' encountered without number (as in F1)"),
            ),
        }
    } else {
        let mut chars = k.chars();
        let ch = chars.next().unwrap();
        assert!(chars.next().is_none());
        Input {
            control: false,
            alt: false,
            key: Key::Key(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kbd_doctest() {
        assert_eq!(
            kbd("a"),
            Input {
                key: Key::Key('a'),
                control: false,
                alt: false
            }
        );
        assert_eq!(
            kbd("C-a"),
            Input {
                key: Key::Key('a'),
                control: true,
                alt: false
            }
        );
        assert_eq!(
            kbd("A-a"),
            Input {
                key: Key::Key('a'),
                control: false,
                alt: true
            }
        );
        assert_eq!(
            kbd("C-A-a"),
            Input {
                key: Key::Key('a'),
                control: true,
                alt: true
            }
        );
        assert_eq!(
            kbd("F1"),
            Input {
                key: Key::Function(1),
                control: false,
                alt: false
            }
        );
        assert_eq!(
            kbd("C-F1"),
            Input {
                key: Key::Function(1),
                control: true,
                alt: false
            }
        );
        assert_eq!(
            kbd("A-F1"),
            Input {
                key: Key::Function(1),
                control: false,
                alt: true
            }
        );
        assert_eq!(
            kbd("C-A-F1"),
            Input {
                key: Key::Function(1),
                control: true,
                alt: true
            }
        );
        assert_eq!(
            kbd("Backspace"),
            Input {
                key: BACKSPACE,
                control: false,
                alt: false
            }
        );
        assert_eq!(
            kbd("C-Backspace"),
            Input {
                key: BACKSPACE,
                control: true,
                alt: false
            }
        );
        assert_eq!(
            kbd("A-Backspace"),
            Input {
                key: BACKSPACE,
                control: false,
                alt: true
            }
        );
        assert_eq!(
            kbd("C-A-Backspace"),
            Input {
                key: BACKSPACE,
                control: true,
                alt: true
            }
        );
    }

    #[test]
    fn input_format_debug_test() {
        assert_eq!(
            "C-A-F1",
            format!(
                "{:?}",
                Input {
                    control: true,
                    alt: true,
                    key: Key::Function(1),
                }
            )
        );
        assert_eq!(
            "C-F1",
            format!(
                "{:?}",
                Input {
                    control: true,
                    alt: false,
                    key: Key::Function(1),
                }
            )
        );
        assert_eq!(
            "A-F1",
            format!(
                "{:?}",
                Input {
                    control: false,
                    alt: true,
                    key: Key::Function(1),
                }
            )
        );
        assert_eq!(
            "F1",
            format!(
                "{:?}",
                Input {
                    control: false,
                    alt: false,
                    key: Key::Function(1),
                }
            )
        );
        assert_eq!(
            "C-A-1",
            format!(
                "{:?}",
                Input {
                    control: true,
                    alt: true,
                    key: Key::Key('1'),
                }
            )
        );
        assert_eq!(
            "C-1",
            format!(
                "{:?}",
                Input {
                    control: true,
                    alt: false,
                    key: Key::Key('1'),
                }
            )
        );
        assert_eq!(
            "A-1",
            format!(
                "{:?}",
                Input {
                    control: false,
                    alt: true,
                    key: Key::Key('1'),
                }
            )
        );
        assert_eq!(
            "1",
            format!(
                "{:?}",
                Input {
                    control: false,
                    alt: false,
                    key: Key::Key('1'),
                }
            )
        );
    }
}
