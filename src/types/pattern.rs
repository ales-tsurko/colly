use std::fmt::Debug;

use crate::clock::CursorPosition;

use serde_derive::Deserialize;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pattern {
    degree: EventStream<Degree>,
    scale: EventStream<Scale>,
    root: EventStream<Root>,
    octave: EventStream<Octave>,
    modulation: EventStream<Modulation>,
}

impl Pattern {
    pub fn events_at_position(
        &mut self,
        position: CursorPosition,
    ) -> Vec<Event<Value>> {
        self.filter(|event| event.position == position).collect()
    }
}

impl Iterator for Pattern {
    type Item = Event<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream<T: Clone + Debug + Default> {
    events: Vec<Event<T>>,
    increment: usize,
}

impl<T: Clone + Debug + Default> From<Vec<Event<T>>> for EventStream<T> {
    fn from(events: Vec<Event<T>>) -> Self {
        EventStream {
            events,
            increment: 0,
        }
    }
}

impl<T: Clone + Debug + Default> Iterator for EventStream<T> {
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
pub struct Event<V: Clone + Debug + Default> {
    value: V,
    position: CursorPosition,
}

impl<T: Clone + Debug + Default> Event<T> {
    pub fn new(value: T, position: CursorPosition) -> Self {
        Event { value, position }
    }
}

impl<T: Clone + Debug + Default> From<(T, CursorPosition)> for Event<T> {
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
pub struct Modulation {
    name: String,
    value: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Scale {
    pub name: String,
    pub pitch_set: Vec<u8>,
}

impl Scale {
    pub fn new(name: &str, pitch_set: &Vec<u8>) -> Self {
        Scale {
            name: name.to_string(),
            pitch_set: pitch_set.clone(),
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale {
            name: "Chromatic".to_string(),
            pitch_set: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pitch(u64),
    Modulation(Modulation),
}

impl Default for Value {
    fn default() -> Self {
        Value::Pitch(60)
    }
}
