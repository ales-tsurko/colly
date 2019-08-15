use super::{Identifier, TypeId, Value};
use std::fmt;

pub trait Function: fmt::Debug + Iterator + FunctionClone + Guide {
    fn identifier(&self) -> Identifier;
    fn arguments(&self) -> Vec<TypeId>;
    fn returns(&self) -> TypeId;
    fn set_arguments(&mut self, arguments: Vec<Value>);
}

pub trait Guide {
    fn description(&self) -> &'static str;
    fn help(&self) -> &'static str;
}

pub trait FunctionClone {
    fn clone_box(&self) -> Box<dyn Function<Item = Value>>;
}

impl<T> FunctionClone for T
where
    T: 'static + Function<Item = Value> + Clone,
{
    fn clone_box(&self) -> Box<dyn Function<Item = Value>> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Function<Item = Value>> {
    fn clone(&self) -> Box<dyn Function<Item = Value>> {
        self.clone_box()
    }
}
