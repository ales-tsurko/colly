use pest::error::{Error, ErrorVariant};
use pest::iterators::{Pair, Pairs};
use pest::RuleType;

pub mod ast;

pub type ParseResult<T> = Result<T, Error<Rule>>;

#[derive(Parser)]
#[grammar = "parser/colly.pest"]
pub struct CollyParser;

impl CollyParser {
    pub fn error<T>(message: &str, pair: &Pair<T>) -> Error<T>
    where
        T: RuleType,
    {
        let variant = ErrorVariant::CustomError {
            message: message.to_string(),
        };
        Error::new_from_span(variant, pair.as_span())
    }

    pub fn rule_error<T>(pair: &Pair<Rule>) -> ParseResult<T> {
        Err(CollyParser::error(
            &format!("Error parsing {:?}", pair.as_rule()),
            &pair,
        ))
    }

    pub fn assert_rule(expected: Rule, pair: &Pair<Rule>) -> ParseResult<()> {
        if pair.as_rule() == expected {
            Ok(())
        } else {
            CollyParser::rule_error(&pair)
        }
    }

    pub fn first_inner_for_pair(pair: Pair<Rule>) -> ParseResult<Pair<Rule>> {
        let span = pair.as_span();
        pair.into_inner().next().ok_or_else(|| {
            Error::new_from_span(
                ErrorVariant::CustomError {
                    message: String::from("Cannot get inner."),
                },
                span,
            )
        })
    }

    pub fn next_pair<'a>(
        pairs: &mut Pairs<'a, Rule>,
        previous: &Pair<'a, Rule>,
    ) -> ParseResult<Pair<'a, Rule>> {
        pairs.next().ok_or_else(|| {
            CollyParser::error("Cannot get next pair.", &previous)
        })
    }
}
