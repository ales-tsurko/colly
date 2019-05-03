use crate::parser::Rule;
use crate::parser_error_from_string_with_pair;
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
};
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Ast<'a>(pub Vec<Statement<'a>>);

impl<'a, 'b: 'a> TryFrom<Pairs<'a, Rule>> for Ast<'b> {
    type Error = Error<Rule>;

    fn try_from(pairs: Pairs<Rule>) -> Result<Self, Self::Error> {
        let mut nodes: Vec<Statement> = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::statement => {
                    let statement = Statement::try_from(pair)?;
                    nodes.push(statement)
                }
                Rule::EOI => continue,
                _ => unreachable!(),
            }
        }
        Ok(Ast(nodes))
    }
}

impl<'a> Ast<'a> {
    fn parse_rule_error<T>(pair: &Pair<Rule>) -> Result<T, Error<Rule>> {
        Err(parser_error_from_string_with_pair(
            &format!("Cannot build statement from {:?}", pair.as_rule()),
            &pair,
        ))
    }
}

pub enum Statement<'a> {
    Assign(Assign<'a>),
    Method,
    Function(FunctionExpression<'a>),
}

impl<'a, 'b: 'a> TryFrom<Pair<'a, Rule>> for Statement<'b> {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> Result<Self, Self::Error> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::assign_statement => Statement::from_assign(&inner),
            _ => Ast::parse_rule_error::<Self>(&inner),
        }
    }
}

impl<'a> Statement<'a> {
    fn from_assign(pair: &Pair<Rule>) -> Result<Self, Error<Rule>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CollyParser;
    use pest::Parser;

    #[test]
    fn test_ast_from_rule() {
        let pairs = CollyParser::parse(Rule::file, "\n(hello world)\n(world hello)\n").unwrap();
        let ast = Ast::try_from(pairs).unwrap();
    }
}

pub enum Assign<'a> {
    Pattern(Expression<'a>, PatternSuperExpression<'a>),
    Variable {
        assignee: Identifier<'a>,
        assignment: SuperExpression<'a>,
    },
    Properties {
        assignee: SuperExpression<'a>,
        assignment: Expression<'a>,
    },
}

pub enum SuperExpression<'a> {
    Expression(Expression<'a>),
    Method {
        caller: Expression<'a>,
        callee: Vec<FunctionExpression<'a>>,
    },
}

pub enum Expression<'a> {
    PropertyGetter {
        assignee: &'a Expression<'a>,
        identifier: Identifier<'a>,
    },
    Boolean(bool),
    Identifier(Identifier<'a>),
    Variable(Identifier<'a>),
    PatternString,
    Number(f64),
    String(&'a str),
    PatternSlot((u64, u64)),
    Track(u64),
    Mixer,
    Properties(Properties<'a>),
    Array(Vec<SuperExpression<'a>>),
    Function(FunctionExpression<'a>),
}

pub struct Properties<'a>(HashMap<Key<'a>, Value<'a>>);
pub struct Key<'a>(Identifier<'a>);
pub enum Value<'a> {
    SuperExpression(SuperExpression<'a>),
    PatternExpression,
}

pub struct Identifier<'a>(pub &'a str);

pub enum FunctionExpression<'a> {
    Function(FunctionCall<'a>),
    FunctionList(Vec<FunctionCall<'a>>),
}

pub struct FunctionCall<'a>(pub Identifier<'a>);

pub enum PatternSuperExpression<'a> {
    ExpressionList(Vec<PatternExpression<'a>>),
    Expression(PatternExpression<'a>),
}

pub struct PatternExpression<'a> {
    pub pattern: Pattern,
    pub inner_method: Option<FunctionExpression<'a>>,
    pub methods: Option<Vec<FunctionExpression<'a>>>,
    pub properties: Option<Properties<'a>>,
}

pub struct Pattern {
    pub inner: Vec<Event>,
}

pub enum Event {
    Chord(Vec<Event>),
    Group(Vec<PatternSymbol>),
    ParenthesisedEvent(Vec<Event>),
}

pub enum PatternSymbol {
    EventMethod(EventMethod),
    Octave(Octave),
    Alteration(Alteration),
    Pitch(u64),
    Pause,
    PatternInput,
    Modulation(Modulation),
}

pub enum EventMethod {
    Tie,
    Dot,
    Multiply,
    Divide,
}

pub enum Octave {
    Up,
    Down,
}

pub enum Alteration {
    Up,
    Down,
}

pub enum Modulation {
    Down,
    Up,
    Crescendo,
    Diminuendo,
    Literal(f64),
}
