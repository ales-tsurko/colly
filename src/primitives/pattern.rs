use super::ValueWrapper;
use crate::clock::Clock;
use crate::parser::ast;

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

impl Pattern {
    fn new_with(ast_pattern: ast::Pattern, clock: &Clock) -> Self {
        let beat_length = clock.beat_length() as f64;
        for group in ast_pattern.0.into_iter() {

        }
        unimplemented!()
    }

    fn interpret_group(group: ast::EventGroup, beat_length: u64) -> Vec<Event> {
        unimplemented!()
    }

    fn interpret_event(event: ast::Event) -> Vec<Event> {
        unimplemented!()
    }
}

impl From<ast::EventGroup> for Vec<Event> {
    fn from(group: ast::EventGroup) -> Self {
        unimplemented!()
    }
}
