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
    cursor: Cursor,
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
    pub fn new(start_position: CursorPosition, cursor: &Cursor) -> Self {
        let mut result = Self {
            degree: EventStream::new(vec![Event::<Degree>::default()], cursor),
            scale: EventStream::new(vec![Event::<Scale>::default()], cursor),
            root: EventStream::new(vec![Event::<Root>::default()], cursor),
            octave: EventStream::new(vec![Event::<Octave>::default()], cursor),
            modulation: EventStream::new(
                vec![Event::<Modulation>::default()],
                cursor,
            ),
            start_position,
            cursor: cursor.clone(),
            is_loop: false,
        };

        result.scale.is_loop = true;
        result.root.is_loop = true;
        result.octave.is_loop = true;

        result
    }

    /// Set if the pattern should loop.
    pub fn set_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
        self.degree.is_loop = is_loop;
        self.modulation.is_loop = is_loop;
    }

    /// Get if pattern is loop.
    pub fn is_loop(&self) -> bool {
        self.is_loop
    }

    /// Reset patern to start position.
    pub fn reset(&mut self) {
        self.cursor.position = (0, 0).into();
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

    fn values_or_default<T>(value: Option<Event<Vec<T>>>) -> Vec<T>
    where
        T: Clone + Default + Debug,
    {
        value
            .or_else(|| Some((vec![T::default()], (0, 0).into()).into()))
            .map(|e| {
                if e.value.is_empty() {
                    vec![T::default()]
                } else {
                    e.value
                }
            })
            .unwrap()
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
        let position = self.cursor.next().unwrap();
        if let Some((degree, modulation)) = self.next_degree_and_modulation() {
            let roots = Pattern::values_or_default(self.root.next());
            let octaves = Pattern::values_or_default(self.octave.next());
            let scales = Pattern::values_or_default(self.scale.next());

            let mut pitches: Vec<Value> = degree
                .value
                .iter()
                .enumerate()
                .map(|(n, d)| {
                    Value::new_pitch(
                        d,
                        &roots[n % roots.len()],
                        &octaves[n % octaves.len()],
                        &scales[n % scales.len()],
                    )
                })
                .collect();
            let mut modulations: Vec<Value> =
                modulation.value.into_iter().map(Value::from).collect();
            
            pitches.append(&mut modulations);
            return Some(Event::new(pitches, position));
        }

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
    pub fn new(events: Vec<Event<T>>, cursor: &Cursor) -> Self {
        let mut result = Self {
            events,
            cursor: cursor.clone(),
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
    state: EventState,
}

impl Degree {
    pub(crate) fn as_pitch_at_scale(&self, scale: &Scale) -> i64 {
        let octave_offset = self.value / scale.pitch_set.len() as u64 * 12;
        (scale.pitch_set[self.value as usize % scale.pitch_set.len()]
            + octave_offset) as i64
            + self.alteration
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pitch(u64, EventState),
    Modulation(String, f64),
}

impl Value {
    pub(crate) fn new_pitch(
        degree: &Degree,
        root: &Root,
        octave: &Octave,
        scale: &Scale,
    ) -> Value {
        let pitch_offset = degree.as_pitch_at_scale(scale);
        let pitch = ((octave.pitch + root.0) as i64 + pitch_offset).max(0);
        Value::Pitch(pitch as u64, degree.state)
    }
}

impl From<Modulation> for Value {
    fn from(value: Modulation) -> Self {
        Value::Modulation(value.name, value.value)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Pitch(60, EventState::On)
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum EventState {
    On,
    Off,
}

impl Default for Degree {
    fn default() -> Self {
        Degree {
            value: 0,
            alteration: 0,
            state: EventState::On,
        }
    }
}

impl From<u64> for Degree {
    fn from(value: u64) -> Self {
        Self {
            value,
            alteration: 0,
            state: EventState::On,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Root(pub(crate) u64);

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Octave {
    pitch: u64,
    octave: u64,
}

impl Octave {
    pub(crate) fn with_octave(octave: u64) -> Self {
        Self {
            pitch: octave * 12,
            octave,
        }
    }

    pub(crate) fn with_pitch(pitch: u64) -> Self {
        Self {
            pitch,
            octave: pitch / 12,
        }
    }

    pub(crate) fn set_as_octave(&mut self, value: u64) {
        self.pitch = 12 * value;
        self.octave = value;
    }

    pub(crate) fn set_as_pitch(&mut self, value: u64) {
        self.octave = value / 12;
        self.pitch = self.octave * 12;
    }

    pub(crate) fn get_octave_number(&self) -> u64 {
        self.octave
    }

    pub(crate) fn get_pitch(&self) -> u64 {
        self.pitch
    }
}

impl Default for Octave {
    fn default() -> Self {
        Self::with_octave(5)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Modulation {
    name: String,
    value: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct Scale {
    pub(crate) name: String,
    pub(crate) pitch_set: Vec<u64>,
}

impl Scale {
    pub(crate) fn new(name: &str, pitch_set: &Vec<u64>) -> Self {
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
            pitch_set: (0..12).collect(),
        }
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
            &Cursor::new(2),
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

        for _ in 0..expected.len() + 3 {
            assert_eq!(None, stream.next());
        }
    }

    #[test]
    fn stream_loop() {
        let mut stream = EventStream::new(
            vec![
                (27, (2, 1).into()).into(),
                (21, (1, 0).into()).into(),
                (15, (0, 0).into()).into(),
            ],
            &Cursor::new(2),
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
            EventStream::new(vec![(0, (0, 0).into()).into()], &Cursor::new(2));
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

        for _ in 0..expected.len() + 3 {
            assert_eq!(None, pattern.next_degree_and_modulation());
        }
    }

    #[test]
    fn octave() {
        let mut octave = Octave::with_octave(5);
        assert_eq!(
            Octave {
                pitch: 60,
                octave: 5
            },
            octave
        );

        octave.set_as_octave(3);
        assert_eq!(
            Octave {
                pitch: 36,
                octave: 3
            },
            octave
        );

        octave.set_as_pitch(75);
        assert_eq!(
            Octave {
                pitch: 72,
                octave: 6
            },
            octave
        );
    }

    #[test]
    fn degree_as_pitch() {
        let scale = Scale::default();
        let degree = Degree::default();
        assert_eq!(0, degree.as_pitch_at_scale(&scale));

        let degree = Degree::from(13);
        assert_eq!(13, degree.as_pitch_at_scale(&scale));

        let mut degree = Degree::from(0);
        degree.alteration = -4;
        assert_eq!(-4, degree.as_pitch_at_scale(&scale));
    }

    #[test]
    fn init_pitch_value() {
        let mut degree = Degree::default();
        let mut root = Root::default();
        let mut octave = Octave::default();
        let scale = Scale::default();
        let value = Value::new_pitch(&degree, &root, &octave, &scale);

        assert_eq!(Value::Pitch(60, EventState::On), value);

        octave.set_as_octave(0);
        root.0 = 0;
        degree.alteration = -5;
        let value = Value::new_pitch(&degree, &root, &octave, &scale);
        assert_eq!(Value::Pitch(0, EventState::On), value);

        octave.set_as_octave(4);
        root.0 = 2;
        degree.value = 3;
        degree.alteration = 1;
        degree.state = EventState::Off;
        let value = Value::new_pitch(&degree, &root, &octave, &scale);
        assert_eq!(Value::Pitch(54, EventState::Off), value);
    }
}
