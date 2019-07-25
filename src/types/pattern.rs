use super::Value;
use crate::clock::{Clock, CursorPosition};
use crate::parser::ast;

#[derive(Debug, Clone, Default)]
pub struct Pattern {
    pub stream: EventStream,
}

#[derive(Debug, Clone, Default)]
pub struct EventStream {
    events: Vec<Event>,
    position: usize,
}

impl From<Vec<Event>> for EventStream {
    fn from(events: Vec<Event>) -> Self {
        EventStream {
            events,
            position: 0
        }
    }
}

impl Iterator for EventStream {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.events.len() {
            let event = self.events[self.position].clone();
            self.position += 1;
            return Some(event);
        }

        self.position = 0;
        None
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    etype: EventType,
    state: EventState,
    position: CursorPosition,
}

#[derive(Debug, Clone)]
pub enum EventState {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Normal(f64),
    Pause,
    Input,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            etype: EventType::Pause,
            state: EventState::Off,
            position: CursorPosition::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventRange {
    start: u64,
    end: u64,
}

