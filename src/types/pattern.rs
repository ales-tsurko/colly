use super::Value;
use crate::clock::{Clock, CursorPosition};
use crate::parser::ast;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pattern {
    pub degree: EventStream<Degree>,
    // pub scale: EventStream<Scale>,
    pub root: EventStream<Root>,
    // pub tuning: EventStream<Tuning>,
    pub octave: EventStream<Octave>,
    pub modulation: HashMap<String, EventStream<Modulation>>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream<T: Clone + std::fmt::Debug + Default> {
    events: Vec<Event<T>>,
    increment: usize,
}

impl<T: Clone + std::fmt::Debug + Default> From<Vec<Event<T>>> for EventStream<T> {
    fn from(events: Vec<Event<T>>) -> Self {
        EventStream {
            events,
            increment: 0,
        }
    }
}

impl<T: Clone + std::fmt::Debug + Default> Iterator for EventStream<T> {
    type Item = Event<T>;

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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event<V: Clone + std::fmt::Debug + Default> {
    value: V,
    position: CursorPosition,
}

impl<T: Clone + std::fmt::Debug + Default> Event<T> {
    pub fn new(
        value: T,
        position: CursorPosition,
    ) -> Self {
        Event {
            value,
            position,
        }
    }
}

impl<T: Clone + std::fmt::Debug + Default> From<(T, CursorPosition)> for Event<T> {
    fn from(value: (T, CursorPosition)) -> Self {
        Event {
            value: value.0,
            position: value.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Degree {
    Note(u64, DegreeState),
    Rest,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DegreeState {
    On,
    Off,
}

impl Default for Degree {
    fn default() -> Self {
        Degree::Note(0, DegreeState::On)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Root(u8);

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Octave(u8);

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Modulation(f64);