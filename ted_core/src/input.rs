/// A single user input event
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    /// A keyboard event
    Key {
        /** The key */
        key: char,
        /** Was control held */
        control: bool,
        /** Was alt held */
        alt: bool,
    },
}

/// The key for the backspace key.
pub const BACKSPACE: char = 127 as char;

use std::fmt;
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Input::Key { key, control, alt } => {
                if control { write!(f, "C-")?; }
                if alt { write!(f, "A-")?; }
                write!(f, "{}", key)
            },
        }
    }
}

/// This macro aids in creation of Key bindings
///
/// # Examples
///
/// ```
/// # use ted_core::{kbd, Input};
/// assert_eq!(kbd!('a'), Input::Key { key: 'a', control: false, alt: false });
/// assert_eq!(kbd!(C-'a'), Input::Key { key: 'a', control: true, alt: false });
/// assert_eq!(kbd!(A-'a'), Input::Key { key: 'a', control: false, alt: true });
/// assert_eq!(kbd!(C-A-'a'), Input::Key { key: 'a', control: true, alt: true });
/// ```
#[macro_export]
macro_rules! kbd {
    (C-A-$key:expr) => (Input::Key { key: $key, control: true, alt: true });
    (C-$key:expr) => (Input::Key { key: $key, control: true, alt: false });
    (A-$key:expr) => (Input::Key { key: $key, control: false, alt: true });
    ($key:expr) => (Input::Key { key: $key, control: false, alt: false });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_1() {
        assert_eq!(format!("{:?}", Input::Key { key: 'a', control: false, alt: false }), "a");
        assert_eq!(format!("{:?}", Input::Key { key: 'a', control: true, alt: false }), "C-a");
        assert_eq!(format!("{:?}", Input::Key { key: 'a', control: false, alt: true }), "A-a");
        assert_eq!(format!("{:?}", Input::Key { key: 'a', control: true, alt: true }), "C-A-a");
    }

    #[test]
    fn kbd_1() {
        assert_eq!(kbd!(C-A-'a'), Input::Key { key: 'a', control: true, alt: true });
        assert_eq!(kbd!('x'), Input::Key { key: 'x', control: false, alt: false });
        assert_eq!(kbd!(A-BACKSPACE), Input::Key { key: BACKSPACE, control: false, alt: true });
    }

    #[test]
    fn kbd_2() {
        assert_eq!(kbd!('a'), Input::Key { key: 'a', control: false, alt: false });
        assert_eq!(kbd!(C-'a'), Input::Key { key: 'a', control: true, alt: false });
        assert_eq!(kbd!(A-'a'), Input::Key { key: 'a', control: false, alt: true });
        assert_eq!(kbd!(C-A-'a'), Input::Key { key: 'a', control: true, alt: true });
    }
}
