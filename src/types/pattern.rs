use std::fmt::Debug;

use crate::clock::{Cursor, CursorPosition};

use serde_derive::Deserialize;

/// Pattern combines several [EventStream](struct.EventStream.html)s
/// and produces [Value](enum.Value.html)s for current [Cursor](../clock/struct.Cursor.html)
/// position on each Pattern::next call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pattern {
    degree: EventStream<Degree>,
    scale: EventStream<Scale>,
    root: EventStream<Root>,
    octave: EventStream<Octave>,
    modulation: EventStream<Modulation>,
    start_position: CursorPosition,
    is_loop: bool,
}

macro_rules! impl_schedule_method {
    ($name:ident, $field:ident, $e_type:ty) => {
            pub(crate) fn $name(&mut self, event: $e_type) {
                self.$field.add_event(event);
            }
    };
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

    pub fn set_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
        self.degree.is_loop = is_loop;
        self.modulation.is_loop = is_loop;
    }

    pub fn is_loop(&self) -> bool {
        self.is_loop
    }

    pub fn reset(&mut self) {
        self.degree.reset();
        self.scale.reset();
        self.root.reset();
        self.octave.reset();
        self.modulation.reset();
    }

    fn next_degree_and_modulation(
        &mut self,
    ) -> Option<(Event<Vec<Degree>>, Event<Vec<Modulation>>)> {
        let degree = self.degree.next();
        let modulation = self.modulation.next();

        if degree.is_none() && modulation.is_none() {
            None
        } else if degree.is_none() && modulation.is_some() {
            self.degree.reset();
            self.degree.next().map(|d| (d, modulation.unwrap()))
        } else if degree.is_some() && modulation.is_none() {
            self.modulation.reset();
            self.modulation.next().map(|m| (degree.unwrap(), m))
        } else {
            Some((degree.unwrap(), modulation.unwrap()))
        }
    }

    impl_schedule_method!(schedule_degree, degree, Event<Degree>);
    impl_schedule_method!(schedule_scale, scale, Event<Scale>);
    impl_schedule_method!(schedule_root, root, Event<Root>);
    impl_schedule_method!(schedult_octave, octave, Event<Octave>);
    impl_schedule_method!(schedule_modulation, modulation, Event<Modulation>);
}

impl Iterator for Pattern {
    type Item = Event<Vec<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        //TODO:
        //Calculate pitch
        //Don't forget to offset position with self.start_position for returning events
        if let Some((degree, modulation)) = self.next_degree_and_modulation() {}

        None
    }
}

/// EventStream is an Iterator which returns values at a specific [Cursor](../clock/struct.Cursor.html)
/// position.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStream<T: Clone + Debug + Default> {
    events: Vec<Event<T>>,
    increment: usize,
    cursor: Cursor,
    is_loop: bool,
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

    /// Add (schedule) [Event](struct.Event.html) to the stream.
    pub fn add_event(&mut self, event: Event<T>) {
        self.events.push(event);
        self.sort();
    }

    /// Get [CursorPosition](../clock/struct.CursorPosition.html) of the last event.
    pub fn last_position(&self) -> Option<CursorPosition> {
        self.events.last().map(|e| e.position)
    }

    pub fn set_loop(&mut self, value: bool) {
        self.is_loop = value;
        self.check_loop();
    }

    pub fn is_loop(&self) -> bool {
        self.is_loop
    }

    /// Reset position to beginning.
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

/// Event is a scheduled value.
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
pub(crate) struct Degree {
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
pub(crate) enum DegreeState {
    On,
    Off,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Root(u8);

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Octave(u8);

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Modulation {
    name: String,
    value: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Scale {
    pub(crate) name: String,
    pub(crate) pitch_set: Vec<u8>,
}

impl Scale {
    pub(crate) fn new(name: &str, pitch_set: &Vec<u8>) -> Self {
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
            pitch_set: (0..11u8).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pitch(u64),
    Modulation(String, f64),
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
        stream.set_loop(true);

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

    #[test]
    fn stream_loop_enabled_afterwards() {
        let mut stream =
            EventStream::new(vec![(0, (0, 0).into()).into()], Cursor::new(2));
        stream.next();
        stream.set_loop(true);

        for _ in 0..5 {
            assert_eq!(
                Some(Event::from((vec![0], (0, 0).into()))),
                stream.next()
            );
        }
    }

    #[test]
    fn pattern_degree_modulation_cycle() {
        let mut pattern = Pattern::default();
        pattern.schedule_degree((Degree::default(), (0, 0).into()).into());
        pattern
            .schedule_modulation((Modulation::default(), (0, 0).into()).into());
        pattern.schedule_degree((Degree::default(), (0, 1).into()).into());

        let mut expected: Vec<(Event<Vec<Degree>>, Event<Vec<Modulation>>)> = vec![
            (
                (vec![Degree::default()], (0, 0).into()).into(),
                (vec![Modulation::default()], (0, 0).into()).into(),
            ),
            (
                (vec![Degree::default()], (0, 1).into()).into(),
                (vec![Modulation::default()], (0, 0).into()).into(),
            ),
        ];

        for _ in 0..expected.len() {
            assert_eq!(
                Some(expected.remove(0)),
                pattern.next_degree_and_modulation()
            );
        }

        assert_eq!(None, pattern.next_degree_and_modulation());
    }
}
