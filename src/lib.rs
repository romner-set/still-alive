use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};

pub mod data;

pub enum Event {
    StartMusic,
    ClearScreen,
    Draw(&'static str), // ref to definition in crate::data::ascii::*
}

pub struct Line {
    pub text: &'static str,
    pub start: Duration,
    pub end: Duration,
    pub event: Option<Event>,
}
impl Line {
    pub const fn new(text: &'static str, start: u64, end: u64) -> Self {
        Self {
            text,
            start: Duration::from_millis(start),
            end: Duration::from_millis(end),
            event: Option::None
        }
    }
    pub const fn event(text: &'static str, start: u64, end: u64, event: Event) -> Self {
        Self {
            text,
            start: Duration::from_millis(start),
            end: Duration::from_millis(end),
            event: Option::Some(event)
        }
    }

    pub fn write(self: &Self, out: &Arc<Mutex<String>>) {
        for c in self.text.chars() {
            out.lock().unwrap().push(c);
            thread::sleep((self.end - self.start) / self.text.len() as u32);
        }
    }
}
