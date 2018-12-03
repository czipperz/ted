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
        /** Was it a function key -- F1 == Function-1 */
        function: bool,
    },
}

/// The key for the backspace key.
pub const BACKSPACE: char = 127 as char;

use std::fmt;
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Input::Key { key, control, alt, function } => {
                if control { write!(f, "C-")?; }
                if alt { write!(f, "A-")?; }
                if function {
                    debug_assert!(key.is_ascii_digit());
                    write!(f, "F")?;
                }
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
    (C-A-F $key:expr) => (Input::Key { key: $key, control: true, alt: true, function: true });
    (C-F $key:expr) => (Input::Key { key: $key, control: true, alt: false, function: true });
    (A-F $key:expr) => (Input::Key { key: $key, control: false, alt: true, function: true });
    (F $key:expr) => (Input::Key { key: $key, control: false, alt: false, function: true });
    (C-A-$key:expr) => (Input::Key { key: $key, control: true, alt: true, function: false });
    (C-$key:expr) => (Input::Key { key: $key, control: true, alt: false, function: false });
    (A-$key:expr) => (Input::Key { key: $key, control: false, alt: true, function: false });
    ($key:expr) => (Input::Key { key: $key, control: false, alt: false, function: false });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kbd_macro_test() {
        assert_eq!(kbd!(C-A-F'1'), Input::Key { key: '1', control: true, alt: true, function: true });
        assert_eq!(kbd!(C-F'1'), Input::Key { key: '1', control: true, alt: false, function: true });
        assert_eq!(kbd!(A-F'1'), Input::Key { key: '1', control: false, alt: true, function: true });
        assert_eq!(kbd!(F'1'), Input::Key { key: '1', control: false, alt: false, function: true });
        assert_eq!(kbd!(C-A-'1'), Input::Key { key: '1', control: true, alt: true, function: false });
        assert_eq!(kbd!(C-'1'), Input::Key { key: '1', control: true, alt: false, function: false });
        assert_eq!(kbd!(A-'1'), Input::Key { key: '1', control: false, alt: true, function: false });
        assert_eq!(kbd!('1'), Input::Key { key: '1', control: false, alt: false, function: false });
    }

    #[test]
    fn kbd_format_debug_test() {
        assert_eq!("C-A-F1", format!("{:?}", Input::Key { key: '1', control: true, alt: true, function: true }));
        assert_eq!("C-F1", format!("{:?}", Input::Key { key: '1', control: true, alt: false, function: true }));
        assert_eq!("A-F1", format!("{:?}", Input::Key { key: '1', control: false, alt: true, function: true }));
        assert_eq!("F1", format!("{:?}", Input::Key { key: '1', control: false, alt: false, function: true }));
        assert_eq!("C-A-1", format!("{:?}", Input::Key { key: '1', control: true, alt: true, function: false }));
        assert_eq!("C-1", format!("{:?}", Input::Key { key: '1', control: true, alt: false, function: false }));
        assert_eq!("A-1", format!("{:?}", Input::Key { key: '1', control: false, alt: true, function: false }));
        assert_eq!("1", format!("{:?}", Input::Key { key: '1', control: false, alt: false, function: false }));
    }
}
