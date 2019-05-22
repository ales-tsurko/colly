use super::ValueWrapper;
use crate::clock::Clock;
use crate::parser::ast;

#[derive(Debug, Clone, Default)]
pub struct Pattern {
    pub stream: EventStream,
}

#[derive(Debug, Clone, Default)]
pub struct EventStream {
    events: Vec<Event>,
    increment: usize,
}

impl From<Vec<Event>> for EventStream {
    fn from(events: Vec<Event>) -> Self {
        EventStream {
            events,
            increment: 0
        }
    }
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
    Normal(f64, EventRange),
    Pause(EventRange),
    Input(EventStream),
}

impl Default for Event {
    fn default() -> Self {
        Event::Pause(EventRange::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventRange {
    start: u64,
    end: u64,
}

