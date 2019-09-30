use std::fmt::Debug;

use crate::clock::{Cursor, CursorPosition, Duration};

use serde_derive::Deserialize;

const DEFAULT_SCALE_NAME: &str = "Chromatic";
const DEFAULT_PITCH_SET: [u64; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

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
    cursor: Cursor,
    start_position: CursorPosition,
    is_loop: bool,
    is_finished: bool,
}

macro_rules! impl_schedule_method {
    ($name:ident, $field:ident, $e_type:ty) => {
        /// Events is scheduled at relative to the pattern position
        /// i.e. the first event's position is (0, 0) and it's
        /// independent of cursor's position.
        pub fn $name(
            &mut self,
            value: $e_type,
            mut position: CursorPosition,
            duration: Duration,
        ) {
            let off_position = position
                .add_position(duration, self.cursor.resolution())
                .sub_position((0,1).into(), self.cursor.resolution());

            self.$field.add_event(Event {
                value: value.clone(),
                position,
                state: EventState::On,
            });

            self.$field.add_event(Event {
                value,
                position: off_position,
                state: EventState::Off,
            });
        }
    };
}

impl Pattern {
    /// To schedule pattern set needed position to the passed cursor.
    pub fn new(mut cursor: Cursor) -> Self {
        let start_position = cursor.position;
        cursor.position = (0, 0).into();
        let mut result = Self {
            degree: EventStream::new(vec![], cursor.resolution()),
            scale: EventStream::new(vec![], cursor.resolution()),
            root: EventStream::new(vec![], cursor.resolution()),
            octave: EventStream::new(vec![], cursor.resolution()),
            modulation: EventStream::new(vec![], cursor.resolution()),
            start_position,
            cursor,
            is_loop: false,
            is_finished: false,
        };

        result.scale.is_loop = true;
        result.scale.fill_gaps = true;

        result.root.is_loop = true;
        result.root.fill_gaps = true;

        result.octave.is_loop = true;
        result.octave.fill_gaps = true;

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

    pub fn start_position(&self) -> CursorPosition {
        self.start_position
    }

    pub fn sort(&mut self) {
        self.degree.sort();
        self.scale.sort();
        self.root.sort();
        self.octave.sort();
        self.modulation.sort();
    }

    #[allow(clippy::type_complexity)]
    fn next_degree_and_modulation(
        &mut self,
    ) -> Option<(Vec<Event<Degree>>, Vec<Event<Modulation>>)> {
        let mut degree = self.degree.next();
        let mut modulation = self.modulation.next();

        if degree.is_none() && modulation.is_none() {
            // return the rest of the beat
            if !self.is_finished && self.cursor.position.tick() != 0 {
                return Some((Vec::new(), Vec::new()));
            }

            self.is_finished = true;
            return None;
        }

        if self.cursor.position.tick() == 0 {
            if degree.is_none() {
                self.degree.reset();
                degree = self.degree.next();
            }

            if modulation.is_none() {
                self.modulation.reset();
                modulation = self.modulation.next();
            }
        }

        Some((degree.unwrap_or_default(), modulation.unwrap_or_default()))
    }

    fn values_or_default<T>(value: Option<Vec<Event<T>>>) -> Vec<Event<T>>
    where
        T: Clone + Default + Debug,
    {
        value
            .or_else(|| Some(vec![Default::default()]))
            .map(|e| {
                if e.is_empty() {
                    vec![Default::default()]
                } else {
                    e
                }
            })
            .unwrap()
    }

    fn next_pitches(
        &mut self,
        degree: Vec<Event<Degree>>,
    ) -> Vec<Event<Value>> {
        let roots = Pattern::values_or_default(self.root.next());
        let octaves = Pattern::values_or_default(self.octave.next());
        let scales = Pattern::values_or_default(self.scale.next());

        degree
            .iter()
            .enumerate()
            .map(|(n, d)| {
                let value = Value::new_pitch(
                    &d.value,
                    &roots[n % roots.len()].value,
                    &octaves[n % octaves.len()].value,
                    &scales[n % scales.len()].value,
                );
                Event::new(value, self.cursor.position, d.state)
            })
            .collect()
    }

    fn init_next_values(
        &mut self,
        degree: Vec<Event<Degree>>,
        modulation: Vec<Event<Modulation>>,
    ) -> Vec<Event<Value>> {
        let pitches = self.next_pitches(degree).into_iter();
        modulation
            .into_iter()
            .map(|m| {
                Event::new(Value::from(m.value), self.cursor.position, m.state)
            })
            .chain(pitches)
            .collect()
    }

    impl_schedule_method!(schedule_degree, degree, Degree);
    impl_schedule_method!(schedule_scale, scale, Scale);
    impl_schedule_method!(schedule_root, root, Root);
    impl_schedule_method!(schedule_octave, octave, Octave);
    impl_schedule_method!(schedule_modulation, modulation, Modulation);
}

impl Iterator for Pattern {
    type Item = Vec<Event<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        let result =
            self.next_degree_and_modulation()
                .map(|(degree, modulation)| {
                    self.init_next_values(degree, modulation)
                });

        self.cursor.next();
        result
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
    /// If this field is set to `true` the EventStream::next will return
    /// a value even if there is no one at the position.
    /// It will either return a previous value or default
    /// if there were previously no any.
    pub fill_gaps: bool,
    gap_value: Vec<Event<T>>,
    is_sorted: bool,
}

impl<T: Clone + Debug + Default> Iterator for EventStream<T> {
    type Item = Vec<Event<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sort();

        if self.events.is_empty()
            || (self.increment >= self.events.len() && !self.is_loop)
        {
            return None;
        }

        let mut result: Self::Item = Vec::new();
        for event in self.events[self.increment..].iter() {
            if event.position == self.cursor.position {
                result.push(event.clone());
                self.increment += 1;
            }

            if event.position > self.cursor.position {
                break;
            }
        }
        self.handle_gaps(&mut result);

        self.cursor.next().unwrap(); // Cursor::next is always Some
        self.check_loop();
        Some(result)
    }
}

impl<T: Clone + Debug + Default> EventStream<T> {
    pub fn new(events: Vec<Event<T>>, resolution: u64) -> Self {
        let mut result = Self {
            events,
            cursor: Cursor::new(resolution),
            gap_value: vec![Default::default()],
            ..Default::default()
        };
        result.sort();
        result
    }

    fn sort(&mut self) {
        if !self.is_sorted {
            self.events.sort_by(|a, b| a.position.cmp(&b.position));
            self.is_sorted = true;
        }
    }

    /// Add (schedule) [Event](struct.Event.html) to the stream.
    pub fn add_event(&mut self, event: Event<T>) {
        self.events.push(event);
        self.is_sorted = false;
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

    fn handle_gaps(&mut self, events: &mut Vec<Event<T>>) {
        if !events.is_empty() {
            self.gap_value = events.clone();
        }

        if self.fill_gaps && events.is_empty() {
            events.append(
                &mut self
                    .gap_value
                    .iter()
                    .cloned()
                    .map(|mut e| {
                        e.position = self.cursor.position;
                        e
                    })
                    .collect(),
            );
        }
    }
}

/// Event is a scheduled value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event<V: Clone + Debug + Default> {
    value: V,
    position: CursorPosition,
    state: EventState,
}

impl<T: Clone + Debug + Default> Event<T> {
    pub fn new(value: T, position: CursorPosition, state: EventState) -> Self {
        Event {
            value,
            position,
            state,
        }
    }
}

impl<T: Clone + Debug + Default> From<(T, CursorPosition)> for Event<T> {
    fn from(value: (T, CursorPosition)) -> Self {
        Event {
            value: value.0,
            position: value.1,
            state: EventState::On,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum EventState {
    On,
    Off,
}

impl Default for EventState {
    fn default() -> Self {
        Self::On
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Pitch(u64),
    Modulation(String, f64),
}

impl Value {
    pub fn new_pitch(
        degree: &Degree,
        root: &Root,
        octave: &Octave,
        scale: &Scale,
    ) -> Value {
        let pitch_offset = degree.as_pitch_at_scale(scale);
        let pitch = ((octave.pitch + root.0) as i64 + pitch_offset).max(0);
        Value::Pitch(pitch as u64)
    }
}

impl From<Modulation> for Value {
    fn from(value: Modulation) -> Self {
        Value::Modulation(value.name, value.value)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Pitch(60)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Degree {
    pub value: u64,
    pub alteration: i64,
}

impl Degree {
    pub fn as_pitch_at_scale(&self, scale: &Scale) -> i64 {
        let octave_offset = self.value / scale.pitch_set.len() as u64 * 12;
        (scale.pitch_set[self.value as usize % scale.pitch_set.len()]
            + octave_offset) as i64
            + self.alteration
    }
}

impl Default for Degree {
    fn default() -> Self {
        Degree {
            value: 0,
            alteration: 0,
        }
    }
}

impl From<u64> for Degree {
    fn from(value: u64) -> Self {
        Self {
            value,
            alteration: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Root(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct Octave {
    pitch: u64,
    octave: u64,
}

impl Octave {
    pub fn with_octave(octave: u64) -> Self {
        Self {
            pitch: octave * 12,
            octave,
        }
    }

    pub fn with_pitch(pitch: u64) -> Self {
        Self {
            pitch,
            octave: pitch / 12,
        }
    }

    pub fn set_as_octave(&mut self, value: u64) {
        self.pitch = 12 * value;
        self.octave = value;
    }

    pub fn set_as_pitch(&mut self, value: u64) {
        self.octave = value / 12;
        self.pitch = self.octave * 12;
    }

    pub fn get_octave_number(&self) -> u64 {
        self.octave
    }

    pub fn get_pitch(&self) -> u64 {
        self.pitch
    }

    pub fn up(&mut self) {
        self.octave = self.octave.saturating_add(1);
        self.set_as_octave(self.octave);
    }

    pub fn down(&mut self) {
        self.octave = self.octave.saturating_sub(1);
        self.set_as_octave(self.octave);
    }
}

impl Default for Octave {
    fn default() -> Self {
        Self::with_octave(5)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Modulation {
    name: String,
    value: f64,
}

impl Modulation {
    pub fn new(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Scale {
    pub name: String,
    pub pitch_set: Vec<u64>,
}

impl Scale {
    pub fn new(name: &str, pitch_set: &[u64]) -> Self {
        Scale {
            name: name.to_string(),
            pitch_set: pitch_set.to_vec(),
        }
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale {
            name: DEFAULT_SCALE_NAME.to_string(),
            pitch_set: DEFAULT_PITCH_SET.to_vec().clone(),
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
            2,
        );

        let mut expected: Vec<Vec<Event<u64>>> = vec![
            vec![(12, (0, 0).into()).into(), (15u64, (0, 0).into()).into()],
            vec![],
            vec![(21, (1, 0).into()).into(), (17u64, (1, 0).into()).into()],
            vec![],
            vec![],
            vec![(27, (2, 1).into()).into()],
        ];

        for _ in 0..expected.len() {
            assert_eq!(Some(expected.remove(0)), stream.next());
        }

        for _ in 0..expected.len() + 3 {
            assert_eq!(None, stream.next());
        }
    }

    #[test]
    fn event_stream_fill_gaps() {
        let mut stream = EventStream::new(
            vec![
                (27, (2, 1).into()).into(),
                (12, (0, 0).into()).into(),
                (21, (1, 0).into()).into(),
                (15, (0, 0).into()).into(),
                (17, (1, 0).into()).into(),
            ],
            2,
        );

        stream.fill_gaps = true;

        let mut expected: Vec<Vec<Event<u64>>> = vec![
            vec![(12, (0, 0).into()).into(), (15, (0, 0).into()).into()],
            vec![(12, (0, 1).into()).into(), (15, (0, 1).into()).into()],
            vec![(21, (1, 0).into()).into(), (17, (1, 0).into()).into()],
            vec![(21, (1, 1).into()).into(), (17, (1, 1).into()).into()],
            vec![(21, (2, 0).into()).into(), (17, (2, 0).into()).into()],
            vec![(27, (2, 1).into()).into()],
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
            2,
        );
        stream.set_loop(true);

        let expected: Vec<Vec<Event<u64>>> = vec![
            vec![(15, (0, 0).into()).into()],
            vec![],
            vec![(21, (1, 0).into()).into()],
            vec![],
            vec![],
            vec![(27, (2, 1).into()).into()],
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
        let mut stream = EventStream::new(vec![(0, (0, 0).into()).into()], 2);
        stream.next();
        stream.set_loop(true);

        for _ in 0..5 {
            assert_eq!(Some(vec![(0, (0, 0).into()).into()]), stream.next());
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

        assert_eq!(Value::Pitch(60), value);

        octave.set_as_octave(0);
        root.0 = 0;
        degree.alteration = -5;
        let value = Value::new_pitch(&degree, &root, &octave, &scale);
        assert_eq!(Value::Pitch(0), value);

        octave.set_as_octave(4);
        root.0 = 2;
        degree.value = 3;
        degree.alteration = 1;
        let value = Value::new_pitch(&degree, &root, &octave, &scale);
        assert_eq!(Value::Pitch(54), value);
    }

    #[test]
    fn pattern_schedule_event() {
        let mut pattern = Pattern::new(Cursor::new(3));
        pattern.schedule_degree(1.into(), (0, 1).into(), (0, 1).into());

        assert_eq!(Vec::<Event<Value>>::new(), pattern.next().unwrap());

        assert_eq!(
            vec![
                Event {
                    value: Value::Pitch(61),
                    position: (0, 1).into(),
                    state: EventState::On,
                },
                Event {
                    value: Value::Pitch(61),
                    position: (0, 1).into(),
                    state: EventState::Off,
                },
            ],
            pattern.next().unwrap()
        );

        assert_eq!(Vec::<Event<Value>>::new(), pattern.next().unwrap());

        assert_eq!(None, pattern.next());
    }

    #[test]
    fn pattern_next_polyrithmic() {
        let mut cursor = Cursor::new(6);
        cursor.position = (2, 1).into();
        let mut pattern = Pattern::new(cursor);
        pattern.schedule_degree(0.into(), (0, 0).into(), (0, 2).into());

        pattern.schedule_degree(1.into(), (1, 0).into(), (0, 4).into());

        pattern.schedule_modulation(
            Modulation::new("v", 0.1),
            (0, 0).into(),
            (0, 3).into(),
        );
        pattern.schedule_modulation(
            Modulation::new("v", 0.2),
            (1, 0).into(),
            (0, 3).into(),
        );
        pattern.schedule_modulation(
            Modulation::new("v", 0.3),
            (2, 0).into(),
            (0, 3).into(),
        );
        pattern.schedule_modulation(
            Modulation::new("v", 0.4),
            (3, 0).into(),
            (0, 3).into(),
        );

        let mut expected: Vec<Vec<Event<Value>>> = vec![
            vec![
                Event::new(
                    Value::Modulation("v".to_string(), 0.1),
                    (0, 0).into(),
                    EventState::On,
                ),
                Event::new(Value::Pitch(60), (0, 0).into(), EventState::On),
            ],
            vec![Event::new(Value::Pitch(60), (0, 1).into(), EventState::Off)],
            vec![Event::new(
                Value::Modulation("v".to_string(), 0.1),
                (0, 2).into(),
                EventState::Off,
            )],
            vec![],
            vec![],
            vec![],
            vec![
                Event::new(
                    Value::Modulation("v".to_string(), 0.2),
                    (1, 0).into(),
                    EventState::On,
                ),
                Event::new(Value::Pitch(61), (1, 0).into(), EventState::On),
            ],
            vec![],
            vec![Event::new(
                Value::Modulation("v".to_string(), 0.2),
                (1, 2).into(),
                EventState::Off,
            )],
            vec![Event::new(Value::Pitch(61), (1, 3).into(), EventState::Off)],
            vec![],
            vec![],
            vec![
                Event::new(
                    Value::Modulation("v".to_string(), 0.3),
                    (2, 0).into(),
                    EventState::On,
                ),
                Event::new(Value::Pitch(60), (2, 0).into(), EventState::On),
            ],
            vec![Event::new(Value::Pitch(60), (2, 1).into(), EventState::Off)],
            vec![Event::new(
                Value::Modulation("v".to_string(), 0.3),
                (2, 2).into(),
                EventState::Off,
            )],
            vec![],
            vec![],
            vec![],
            vec![
                Event::new(
                    Value::Modulation("v".to_string(), 0.4),
                    (3, 0).into(),
                    EventState::On,
                ),
                Event::new(Value::Pitch(61), (3, 0).into(), EventState::On),
            ],
            vec![],
            vec![Event::new(
                Value::Modulation("v".to_string(), 0.4),
                (3, 2).into(),
                EventState::Off,
            )],
            vec![Event::new(Value::Pitch(61), (3, 3).into(), EventState::Off)],
            vec![],
            vec![],
            // vec![],
        ];

        for _ in 0..expected.len() {
            assert_eq!(expected.remove(0), pattern.next().unwrap());
        }

        for _ in 0..3 {
            assert_eq!(None, pattern.next());
        }
    }
}
