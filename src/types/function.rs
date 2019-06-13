use super::{Identifier, ValueWrapper, TypeId};
use std::fmt;

pub trait Function: fmt::Debug + Iterator + FunctionClone + Guide {
    fn identifier(&self) -> Identifier;
    fn arguments(&self) -> Vec<TypeId>;
    fn returns(&self) -> TypeId;
    fn set_arguments(&mut self, arguments: Vec<ValueWrapper>);
}

pub trait Guide {
    fn description(&self) -> &'static str;
    fn help(&self) -> &'static str;
}

pub trait FunctionClone {
    fn clone_box(&self) -> Box<Function<Item = ValueWrapper>>;
}

impl<'a, T> FunctionClone for T
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
