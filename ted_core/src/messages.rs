use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct Messages {
    messages: VecDeque<String>,
    message_display_time: Option<Instant>,
}

impl Messages {
    pub fn new() -> Self {
        Messages {
            messages: VecDeque::new(),
            message_display_time: None,
        }
    }
    
    pub fn add<S: ToString>(&mut self, message: S) {
        self.messages.push_back(message.to_string());
    }

    pub fn poll(&mut self) -> Option<&str> {
        if let Some(time) = self.message_display_time {
            if time.elapsed() > Duration::from_secs(10) {
                self.messages.pop_front();
                self.message_display_time = None;
            }
        }
        if self.message_display_time.is_none() {
            if !self.messages.is_empty() {
                self.message_display_time = Some(Instant::now());
            }
        }
        self.messages.front().map(|x| x.as_ref())
    }
}
