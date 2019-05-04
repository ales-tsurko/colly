use crate::parser::Rule;
use crate::parser_error_from_string_with_pair;
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

type ParseResult<T> = Result<T, Error<Rule>>;

pub struct Ast(pub Vec<Statement>);

impl<'a> TryFrom<Pairs<'a, Rule>> for Ast {
    type Error = Error<Rule>;

    fn try_from(pairs: Pairs<Rule>) -> ParseResult<Self> {
        let mut nodes: Vec<Statement> = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::Statement => nodes.push(pair.try_into()?),
                Rule::EOI => continue,
                _ => unreachable!(),
            }
        }
        Ok(Ast(nodes))
    }
}

impl Ast {
    pub fn parse_rule_error<T>(pair: &Pair<Rule>) -> ParseResult<T> {
        Err(parser_error_from_string_with_pair(
            &format!("Cannot build statement from {:?}", pair.as_rule()),
            &pair,
        ))
    }

    pub fn assert_rule(expected: Rule, pair: &Pair<Rule>) -> ParseResult<()> {
        if pair.as_rule() == expected {
            Ok(())
        } else {
            Ast::parse_rule_error(&pair)
        }
    }
}


pub enum Statement {
    Assign(Assign),
    Method,
    Function(FunctionExpression),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::FunctionExpression => Statement::from_function_expression(inner),
            Rule::MethodCall => Statement::from_method_call(inner),
            Rule::AssignStatement => Statement::from_assign(inner),
            _ => Ast::parse_rule_error::<Self>(&inner),
        }
    }
}

impl Statement {
    fn from_function_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::Function(pair.try_into()?))
    }

    fn from_method_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::Method)
    }

    fn from_assign(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::Assign(pair.try_into()?))
    }
}

pub enum FunctionExpression {
    Function(FunctionCall),
    FunctionList(Vec<FunctionCall>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for FunctionExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }
}

pub struct FunctionCall(pub Identifier);

pub enum Assign {
    Pattern(Expression, PatternSuperExpression),
    Variable {
        assignee: Identifier,
        assignment: SuperExpression,
    },
    Properties {
        assignee: SuperExpression,
        assignment: Expression,
    },
}

impl<'a> TryFrom<Pair<'a, Rule>> for Assign {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
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
        let pairs = CollyParser::parse(Rule::File, "\nhello world\n").unwrap();
        let _ast = Ast::try_from(pairs).unwrap();
    }
}

pub enum SuperExpression {
    Expression(Expression),
    Method {
        caller: Expression,
        callee: Vec<FunctionExpression>,
    },
}

pub enum Expression {
    PropertyGetter {
        assignee: Box<Expression>,
        identifier: Identifier,
    },
    Boolean(bool),
    Identifier(Identifier),
    Variable(Identifier),
    PatternString,
    Number(f64),
    String(String),
    PatternSlot((u64, u64)),
    Track(u64),
    Mixer,
    Properties(Properties),
    Array(Vec<SuperExpression>),
    Function(FunctionExpression),
}

pub struct Properties(HashMap<Key, Value>);
pub struct Key(Identifier);
pub enum Value {
    SuperExpression(SuperExpression),
    PatternExpression,
}

pub struct Identifier(pub String);

pub enum PatternSuperExpression {
    ExpressionList(Vec<PatternExpression>),
    Expression(PatternExpression),
}

pub struct PatternExpression {
    pub pattern: Pattern,
    pub inner_method: Option<FunctionExpression>,
    pub methods: Option<Vec<FunctionExpression>>,
    pub properties: Option<Properties>,
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
