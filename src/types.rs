mod function;
mod mixer;
mod pattern;

pub use function::*;
pub use mixer::*;
pub use pattern::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Value<T> {
    inner: T,
}

#[derive(Debug)]
pub enum ValueWrapper {
    Identifier(Value<Identifier>),
    Boolean(Value<bool>),
    Number(Value<f64>),
    String(Value<String>),
    Properties(Value<Properties>),
    Array(Value<Vec<ValueWrapper>>),
    Function(Box<Function>),
    Patter(Value<Pattern>),
    Mixer(Value<Mixer>),
    Track(Value<Track>),
    Slot(Value<Slot>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug)]
pub struct Properties(pub HashMap<Identifier, ValueWrapper>);