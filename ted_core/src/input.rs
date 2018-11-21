#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    Key { c: char, control: bool, alt: bool },
}

pub const BACKSPACE: char = 127 as char;

use std::fmt;
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Input::Key { c, control, alt } => {
                if control { write!(f, "C-")?; }
                if alt { write!(f, "A-")?; }
                write!(f, "{}", c)
            },
        }
    }
}

#[macro_export]
macro_rules! kbd {
    (C-A-$c:expr) => (Input::Key { c: $c, control: true, alt: true });
    (C-$c:expr) => (Input::Key { c: $c, control: true, alt: false });
    (A-$c:expr) => (Input::Key { c: $c, control: false, alt: true });
    ($c:expr) => (Input::Key { c: $c, control: false, alt: false });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_1() {
        assert_eq!(format!("{:?}", Input::Key { c: 'a', control: false, alt: false }), "a");
        assert_eq!(format!("{:?}", Input::Key { c: 'a', control: true, alt: false }), "C-a");
        assert_eq!(format!("{:?}", Input::Key { c: 'a', control: false, alt: true }), "A-a");
        assert_eq!(format!("{:?}", Input::Key { c: 'a', control: true, alt: true }), "C-A-a");
    }

    #[test]
    fn kbd_1() {
        assert_eq!(kbd!(C-A-'a'), Input::Key { c: 'a', control: true, alt: true });
        assert_eq!(kbd!('x'), Input::Key { c: 'x', control: false, alt: false });
        assert_eq!(kbd!(A-BACKSPACE), Input::Key { c: BACKSPACE, control: false, alt: true });
    }
}
