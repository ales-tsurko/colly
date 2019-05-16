use super::{Identifier, ValueWrapper};
use std::fmt::Debug;

pub trait Function: Debug + Iterator {
    fn identifier(&self) -> Identifier;
    fn arguments(&self) -> Vec<Argument>;
    fn set_arguments(&mut self, arguments: Vec<ValueWrapper>);
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
