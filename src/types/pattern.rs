use super::Value;
use crate::clock::{Clock, CursorPosition};
use crate::parser::ast;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pattern {
    pub stream: EventStream,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream {
    events: Vec<Event>,
    position: usize,
}

impl From<Vec<Event>> for EventStream {
    fn from(events: Vec<Event>) -> Self {
        EventStream {
            events,
            position: 0,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    etype: EventType,
    state: EventState,
    position: CursorPosition,
}

impl Event {
    pub fn new(
        etype: EventType,
        state: EventState,
        position: CursorPosition,
    ) -> Self {
        Event {
            etype,
            state,
            position,
        }
    }
}

impl From<(EventType, EventState, CursorPosition)> for Event {
    fn from(value: (EventType, EventState, CursorPosition)) -> Self {
        Event {
            etype: value.0,
            state: value.1,
            position: value.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventState {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq)]
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
