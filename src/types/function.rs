use super::{Identifier, ValueWrapper};
use std::fmt::Debug;

pub trait Function: Debug {
    fn identifier(&self) -> Identifier;
    fn arguments(&self) -> Vec<Argument>;
    fn next(&mut self, arguments: Vec<ValueWrapper>) -> Option<ValueWrapper>;
}

pub enum Argument {
    Identifier,
    Boolean,
    Number,
    Pattern,
    String,
    Properties,
    Array,
    Function,
    Track,
    Slot,
}
