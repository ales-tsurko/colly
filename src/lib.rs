#[macro_use]
extern crate pest_derive;
pub mod parser;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::RuleType;

fn parser_error_from_string_with_pair<T>(message: &str, pair: &Pair<T>)
    -> Error<T>
    where T: RuleType
{
    let variant = ErrorVariant::CustomError {
        message: message.to_string(),
    };
    Error::new_from_span(variant, pair.as_span())
}
