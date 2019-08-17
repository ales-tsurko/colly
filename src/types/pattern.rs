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
    pub fn new(start_position: CursorPosition)-> Self {
        Self { start_position, ..Default::default() }
    }
}

impl Iterator for Pattern {
    type Item = Event<Vec<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO:
        //Check if there are events at the self.cursor.next() position
        //Calculate pitch
        //Update position with self.start_position for returning events
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream<T: Clone + Debug + Default> {
    events: Vec<Event<T>>,
    cursor: Cursor,
}

impl<T: Clone + Debug + Default> EventStream<T> {
    pub fn new(events: Vec<Event<T>>, cursor: Cursor) -> Self {
        Self { events, cursor }
    }
}

impl<T: Clone + Debug + Default> Iterator for EventStream<T> {
    type Item = Event<Vec<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.events.is_empty() {
            return None;
        }

        let position = self.cursor.position;
        let mut values: Vec<T> = Vec::new();
        while position == self.events[0].position {
            values.push(self.events.remove(0).value);
            if self.events.is_empty() {
                break;
            }
        }

        self.cursor.next().unwrap(); // Cursor::next is always Some
        Some((values, position).into())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_stream_iterator() {
        let mut stream = EventStream::new(
            vec![
                (12, (0, 0).into()).into(),
                (15, (0, 0).into()).into(),
                (17, (1, 0).into()).into(),
                (21, (1, 0).into()).into(),
                (27, (2, 1).into()).into(),
            ],
            Cursor::new(2),
        );
        let mut expected: Vec<Event<Vec<u64>>> = vec![
            (vec![12, 15], (0, 0).into()).into(),
            (vec![], (0, 1).into()).into(),
            (vec![17, 21], (1, 0).into()).into(),
            (vec![], (1, 1).into()).into(),
            (vec![], (2, 0).into()).into(),
            (vec![27], (2, 1).into()).into(),
        ];

        for _ in 0..expected.len() {
            assert_eq!(Some(expected.remove(0)), stream.next());
        }
    }
}
