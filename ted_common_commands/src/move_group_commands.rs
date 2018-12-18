use move_commands::forward_word;
use parking_lot::Mutex;
use std::sync::Arc;
use ted_core::*;

#[derive(Debug)]
pub struct ForwardGroupCommand;

/// Construct a [`ForwardGroupCommand`].
///
/// [`ForwardGroupCommand`]: struct.ForwardGroupCommand.html
pub fn forward_group_command() -> Arc<ForwardGroupCommand> {
    Arc::new(ForwardGroupCommand)
}

impl Command for ForwardGroupCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        let selected_window = &mut *selected_window;
        let buffer = selected_window.buffer.lock();
        let new_location = forward_group(&buffer, selected_window.cursor.get(), 1);
        selected_window.cursor.set(&buffer, new_location);
        Ok(())
    }
}

#[derive(Debug)]
pub struct BackwardGroupCommand;

/// Construct a [`BackwardGroupCommand`].
///
/// [`BackwardGroupCommand`]: struct.BackwardGroupCommand.html
pub fn backward_group_command() -> Arc<BackwardGroupCommand> {
    Arc::new(BackwardGroupCommand)
}

impl Command for BackwardGroupCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        let selected_window = &mut *selected_window;
        let buffer = selected_window.buffer.lock();
        let new_location = forward_group(&buffer, selected_window.cursor.get(), -1);
        selected_window.cursor.set(&buffer, new_location);
        Ok(())
    }
}

pub fn forward_group(buffer: &Buffer, mut ret_location: usize, times: isize) -> usize {
    let mut location = ret_location;
    if times < 0 {
        for _ in times..0 {
            if location == 0 {
                return ret_location;
            }
            while buffer.get(location - 1).unwrap().is_whitespace() {
                location -= 1;
                if location == 0 {
                    return ret_location;
                }
            }
            let c = buffer.get(location - 1).unwrap();
            if c == ')' || c == ']' || c == '}' {
                let mut depth = 0;
                loop {
                    let c = buffer.get(location - 1).unwrap();
                    if c == ')' || c == ']' || c == '}' {
                        depth += 1;
                    } else if c == '(' || c == '[' || c == '{' {
                        depth -= 1;
                    }
                    location -= 1;
                    if depth == 0 {
                        break;
                    }
                    if location == 0 {
                        return ret_location;
                    }
                }
            } else if c == '(' || c == '[' || c == '{' {
                return ret_location;
            } else {
                location = forward_word(buffer, location, -1);
            }
            ret_location = location;
        }
    } else {
        let buffer_len = buffer.len();
        for _ in 0..times {
            if location + 1 >= buffer_len {
                return ret_location;
            }
            while buffer.get(location).unwrap().is_whitespace() {
                location += 1;
                if location == buffer_len {
                    return ret_location;
                }
            }
            let c = buffer.get(location).unwrap();
            if c == '(' || c == '[' || c == '{' {
                let mut depth = 0;
                loop {
                    let c = buffer.get(location).unwrap();
                    if c == '(' || c == '[' || c == '{' {
                        depth += 1;
                    } else if c == ')' || c == ']' || c == '}' {
                        depth -= 1;
                    }
                    location += 1;
                    if depth == 0 {
                        break;
                    }
                    if location == buffer_len {
                        return ret_location;
                    }
                }
            } else if c == ')' || c == ']' || c == '}' {
                return ret_location;
            } else {
                location = forward_word(buffer, location, 1);
            }
            ret_location = location;
        }
    }
    location
}

#[derive(Debug)]
pub struct UpGroupCommand;

/// Construct a [`UpGroupCommand`].
///
/// [`UpGroupCommand`]: struct.UpGroupCommand.html
pub fn up_group_command() -> Arc<UpGroupCommand> {
    Arc::new(UpGroupCommand)
}

impl Command for UpGroupCommand {
    fn execute(&self, state: Arc<Mutex<State>>) -> Result<(), String> {
        let selected_window = state.lock().display.selected_window();
        let mut selected_window = selected_window.lock();
        let selected_window = &mut *selected_window;
        let buffer = selected_window.buffer.lock();
        let new_location = up_group(&buffer, selected_window.cursor.get(), 1);
        selected_window.cursor.set(&buffer, new_location);
        Ok(())
    }
}

pub fn up_group(buffer: &Buffer, mut location: usize, times: usize) -> usize {
    let mut ret_location = location;
    if location == 0 {
        return ret_location;
    }
    for _ in 0..times {
        let mut depth = 1;
        loop {
            let c = buffer.get(location - 1).unwrap();
            if c == ')' || c == ']' || c == '}' {
                depth += 1;
            } else if c == '(' || c == '[' || c == '{' {
                depth -= 1;
            }
            location -= 1;
            if depth == 0 {
                break;
            }
            if location == 0 {
                return ret_location;
            }
        }
        ret_location = location;
    }
    location
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_group_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " (abc) ").unwrap();
        assert_eq!(forward_group(&buffer, 0, 1), 6);
        assert_eq!(forward_group(&buffer, 1, 1), 6);
    }

    #[test]
    fn backward_group_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " (abc) ").unwrap();
        assert_eq!(forward_group(&buffer, 6, -1), 1);
        assert_eq!(forward_group(&buffer, 7, -1), 1);
    }

    #[test]
    fn forward_group_identifier_underscores() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " ab_cd ").unwrap();
        assert_eq!(forward_group(&buffer, 0, 1), 6);
        assert_eq!(forward_group(&buffer, 1, 1), 6);
    }

    #[test]
    fn backward_group_identifier_underscores() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " ab_cd ").unwrap();
        assert_eq!(forward_group(&buffer, 6, -1), 1);
        assert_eq!(forward_group(&buffer, 7, -1), 1);
    }

    #[test]
    fn forward_group_nested_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " ((oh) [boy]) ").unwrap();
        assert_eq!(forward_group(&buffer, 0, 1), 13);
        assert_eq!(forward_group(&buffer, 1, 1), 13);
        assert_eq!(forward_group(&buffer, 1, 2), 13);
        assert_eq!(forward_group(&buffer, 2, 1), 6);
        assert_eq!(forward_group(&buffer, 2, 2), 12);
    }

    #[test]
    fn backward_group_nested_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, " ((oh) [boy]) ").unwrap();
        assert_eq!(forward_group(&buffer, 14, -1), 1);
        assert_eq!(forward_group(&buffer, 13, -1), 1);
        assert_eq!(forward_group(&buffer, 13, -2), 1);
        assert_eq!(forward_group(&buffer, 6, -1), 2);
        assert_eq!(forward_group(&buffer, 12, -2), 2);
    }

    #[test]
    fn up_group_nested_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "((oh man))").unwrap();
        assert_eq!(up_group(&buffer, 5, 1), 1);
        assert_eq!(up_group(&buffer, 5, 2), 0);
    }

    #[test]
    fn up_group_multiple_nested_parens() {
        let mut buffer = Buffer::new("*scratch*".into());
        buffer.insert_str(0, "((oh) (man))").unwrap();
        assert_eq!(up_group(&buffer, 7, 1), 6);
        assert_eq!(up_group(&buffer, 7, 2), 0);
    }
}
