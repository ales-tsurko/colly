#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pattern {
    stream: EventStream,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventStream;

impl Iterator for EventStream {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Event {
    value: f64,
    duration: f64,
}

//TODO
// impl TryFrom<PatternSuperExpression> for EventStream
