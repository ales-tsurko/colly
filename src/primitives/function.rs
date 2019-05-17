use super::{Identifier, ValueWrapper};
use std::fmt::Debug;

pub trait Function: Debug + Iterator + FunctionClone {
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

pub trait FunctionClone {
    fn clone_box(&self) -> Box<Function<Item = ValueWrapper>>;
}

impl<T> FunctionClone for T
where
    T: 'static + Function<Item = ValueWrapper> + Clone,
{
    fn clone_box(&self) -> Box<Function<Item = ValueWrapper>> {
        Box::new(self.clone())
    }
}

impl Clone for Box<Function<Item = ValueWrapper>> {
    fn clone(&self) -> Box<Function<Item = ValueWrapper>> {
        self.clone_box()
    }
}
