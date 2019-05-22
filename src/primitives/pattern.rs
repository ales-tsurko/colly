use super::ValueWrapper;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct Pattern {
    stream: EventStream,
}

#[derive(Debug, Clone, Default)]
pub struct EventStream {
    events: Vec<Event>,
    increment: usize,
}

impl Iterator for EventStream {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.increment < self.events.len() {
            let event = self.events[self.increment].clone();
            self.increment += 1;
            return Some(event);
        }

        self.increment = 0;
        None
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Normal(f64, Duration),
    Pause(Duration),
    Input(ValueWrapper),
}

impl Default for Event {
    fn default() -> Self {
        Event::Pause(Duration::default())
    }
}