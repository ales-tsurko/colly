use std::collections::HashSet;
use std::fmt::Debug;

use crate::clock::{Cursor, CursorPosition};

use serde_derive::Deserialize;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pattern {
    degree: EventStream<Degree>,
    scale: EventStream<Scale>,
    root: EventStream<Root>,
    octave: EventStream<Octave>,
    modulation: EventStream<Modulation>,
    start_position: CursorPosition,
}

impl Pattern {
    pub fn new(start_position: CursorPosition) -> Self {
        let mut result = Self {
            start_position,
            ..Default::default()
        };

        result.scale.is_loop = true;
        result.root.is_loop = true;
        result.octave.is_loop = true;

        result
    }
}

impl Iterator for Pattern {
    type Item = Event<Vec<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO:
        //Calculate pitch
        //Don't forget to offset position with self.start_position for returning events
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream<T: Clone + Debug + Default> {
    events: Vec<Event<T>>,
    increment: usize,
    cursor: Cursor,
    pub is_loop: bool,
}

impl<T: Clone + Debug + Default> EventStream<T> {
    pub fn new(events: Vec<Event<T>>, cursor: Cursor) -> Self {
        let mut result = Self {
            events,
            cursor,
            ..Default::default()
        };
        result.sort();
        result
    }

    fn sort(&mut self) {
        self.events.sort_by(|a, b| a.position.cmp(&b.position));
    }

    pub fn add_event(&mut self, event: Event<T>) {
        self.events.push(event);
        self.sort();
    }

    pub fn last_position(&self) -> Option<CursorPosition> {
        self.events.last().map(|e| e.position)
    }

    pub fn reset(&mut self) {
        self.increment = 0;
        self.cursor.position = (0, 0).into();
    }

    fn check_loop(&mut self) {
        if self.is_loop && self.increment >= self.events.len() {
            self.reset();
        }
    }
}

impl<T: Clone + Debug + Default> Iterator for EventStream<T> {
    type Item = Event<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.events.is_empty()
            || (self.increment >= self.events.len() && !self.is_loop)
        {
            return None;
        }

        let mut result: Self::Item = (vec![], self.cursor.position).into();
        while self.increment < self.events.len()
            && self.cursor.position == self.events[self.increment].position
        {
            result.value.push(self.events[self.increment].value.clone());
            self.increment += 1;
        }

        self.cursor.next().unwrap(); // Cursor::next is always Some
        self.check_loop();
        Some(result)
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
pub struct Degree {
    value: u64,
    alteration: i64,
    state: DegreeState,
}

impl Default for Degree {
    fn default() -> Self {
        Degree {
            value: 0,
            alteration: 0,
            state: DegreeState::On,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DegreeState {
    On,
    Off,
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
    pub pitch_set: HashSet<u8>,
}

impl Scale {
    pub fn new(name: &str, pitch_set: &HashSet<u8>) -> Self {
        Scale {
            name: name.to_string(),
            pitch_set: pitch_set.clone(),
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        let mut set: HashSet<u8> = HashSet::new();
        for n in 0..11u8 {
            set.insert(n);
        }
        Scale {
            name: "Chromatic".to_string(),
            pitch_set: set,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_stream() {
        let mut stream = EventStream::new(
            vec![
                (27, (2, 1).into()).into(),
                (12, (0, 0).into()).into(),
                (21, (1, 0).into()).into(),
                (15, (0, 0).into()).into(),
                (17, (1, 0).into()).into(),
            ],
            Cursor::new(2),
        );

        let mut expected: Vec<Event<Vec<u64>>> = vec![
            (vec![12, 15], (0, 0).into()).into(),
            (vec![], (0, 1).into()).into(),
            (vec![21, 17], (1, 0).into()).into(),
            (vec![], (1, 1).into()).into(),
            (vec![], (2, 0).into()).into(),
            (vec![27], (2, 1).into()).into(),
        ];

        for _ in 0..expected.len() {
            assert_eq!(Some(expected.remove(0)), stream.next());
        }

        assert_eq!(None, stream.next());
    }

    #[test]
    fn stream_loop() {
        let mut stream = EventStream::new(
            vec![
                (27, (2, 1).into()).into(),
                (21, (1, 0).into()).into(),
                (15, (0, 0).into()).into(),
            ],
            Cursor::new(2),
        );
        stream.is_loop = true;

        let expected: Vec<Event<Vec<u64>>> = vec![
            (vec![15], (0, 0).into()).into(),
            (vec![], (0, 1).into()).into(),
            (vec![21], (1, 0).into()).into(),
            (vec![], (1, 1).into()).into(),
            (vec![], (2, 0).into()).into(),
            (vec![27], (2, 1).into()).into(),
        ];

        for n in 0..expected.len() * 3 {
            assert_eq!(
                Some(expected[n % expected.len()].clone()),
                stream.next()
            );
        }
    }
}
