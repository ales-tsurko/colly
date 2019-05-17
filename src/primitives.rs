mod function;
mod mixer;
mod pattern;

pub use function::*;
pub use mixer::*;
pub use pattern::*;
use std::collections::HashMap;

type PremitiveResult<T> = Result<T, PrimitiveError>;

#[derive(Debug, Clone, PartialEq)]
pub struct Value<T>(pub T);

#[derive(Debug, Clone)]
pub enum ValueWrapper {
    Identifier(Value<Identifier>),
    Boolean(Value<bool>),
    Number(Value<f64>),
    String(Value<String>),
    Properties(Value<Properties>),
    Array(Value<Vec<ValueWrapper>>),
    Function(Box<Function<Item = ValueWrapper>>),
    Pattern(Value<Pattern>),
    Mixer(Value<Mixer>),
    Track(Value<Track>),
    Slot(Value<Slot>),
    Void(Value<()>),
    Nothing,
}

macro_rules! impl_from_for_value_wrapper {
    ($from:ty, $item:ident) => {
        impl From<$from> for ValueWrapper {
            fn from(value: $from) -> Self {
                ValueWrapper::$item(Value(value))
            }
        }
    };
}

impl_from_for_value_wrapper!(Identifier, Identifier);
impl_from_for_value_wrapper!(bool, Boolean);
impl_from_for_value_wrapper!(f64, Number);
impl_from_for_value_wrapper!(String, String);
impl_from_for_value_wrapper!(Properties, Properties);
impl_from_for_value_wrapper!(Vec<ValueWrapper>, Array);
impl_from_for_value_wrapper!(Pattern, Pattern);
impl_from_for_value_wrapper!(Mixer, Mixer);
impl_from_for_value_wrapper!(Track, Track);
impl_from_for_value_wrapper!(Slot, Slot);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Properties(pub HashMap<Identifier, ValueWrapper>);

pub trait HasProperties {
    fn property(&self, key: &Identifier) -> Option<ValueWrapper>;
    fn set_property(
        &mut self,
        key: &Identifier,
        value: ValueWrapper,
    ) -> PremitiveResult<()>;
}

#[derive(Debug, Fail)]
pub enum PrimitiveError {
    #[fail(
        display = "Cannot set property {} for {}: {}",
        property_name, assignee_name, cause
    )]
    SetProperty {
        property_name: String,
        assignee_name: String,
        cause: String,
    },
}
