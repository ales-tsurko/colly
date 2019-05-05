#[cfg(test)]
mod tests;
use crate::parser::Rule;
use crate::parser_error_from_string_with_pair;
use pest::{
    error::{Error, ErrorVariant},
    iterators::{Pair, Pairs},
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

type ParseResult<T> = Result<T, Error<Rule>>;

pub struct Ast(pub Vec<Statement>);

impl<'a> TryFrom<Pairs<'a, Rule>> for Ast {
    type Error = Error<Rule>;

    fn try_from(pairs: Pairs<Rule>) -> ParseResult<Self> {
        let raw_statements: ParseResult<Vec<Pair<Rule>>> = pairs
            .filter(|pair| pair.as_rule() == Rule::Statement)
            .map(Ast::inner_for_pair)
            .collect();
        let statements: ParseResult<Vec<Statement>> = raw_statements?
            .into_iter()
            .map(Statement::try_from)
            .collect();
        Ok(Ast(statements?))
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

    pub fn inner_for_pair(pair: Pair<Rule>) -> ParseResult<Pair<Rule>> {
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
}

//
pub enum Statement {
    SuperExpression(SuperExpression),
    Assign(Assign),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::SuperExpression => Statement::from_super_expression(pair),
            Rule::AssignStatement => Statement::from_assign(pair),
            _ => Ast::parse_rule_error::<Self>(&pair),
        }
    }
}

impl Statement {
    fn from_super_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = Ast::inner_for_pair(pair)?;
        Ok(Statement::SuperExpression(inner.try_into()?))
    }

    fn from_assign(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::Assign(pair.try_into()?))
    }
}

//
pub enum SuperExpression {
    Expression(Expression),
    Method(MethodCall),
}

impl<'a> TryFrom<Pair<'a, Rule>> for SuperExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::Expression => SuperExpression::from_expression(pair),
            Rule::MethodCall => SuperExpression::from_method_call(pair),
            _ => Ast::parse_rule_error::<Self>(&pair),
        }
    }
}

impl SuperExpression {
    pub fn from_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = Ast::inner_for_pair(pair)?;
        Ok(SuperExpression::Expression(inner.try_into()?))
    }

    pub fn from_method_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Method(pair.try_into()?))
    }
}

//
pub enum Expression {
    PropertyGetter(PropertyGetter),
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

impl<'a> TryFrom<Pair<'a, Rule>> for Expression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::PropertyGetter => Expression::from_property_getter(pair),
            Rule::Boolean => Expression::from_boolean(pair),
            Rule::Identifier => Expression::from_identifier(pair),
            Rule::Variable => Expression::from_variable(pair),
            Rule::PatternString => Expression::from_pattern_string(pair),
            Rule::Number => Expression::from_number(pair),
            Rule::String => Expression::from_string(pair),
            Rule::PatternSlot => Expression::from_pattern_slot(pair),
            Rule::Track => Expression::from_track(pair),
            Rule::Mixer => Expression::from_mixer(pair),
            Rule::Properties => Expression::from_properties(pair),
            Rule::FunctionExpression => Expression::from_function_expression(pair),
            Rule::Array => Expression::from_array(pair),
            _ => Ast::parse_rule_error::<Self>(&pair),
        }
    }
}

impl Expression {
    pub fn from_property_getter(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = Ast::inner_for_pair(pair)?;
        dbg!(&inner);
        Ok(Expression::PropertyGetter(inner.try_into()?))
    }

    pub fn from_boolean(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_identifier(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_variable(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_pattern_string(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_number(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_string(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_pattern_slot(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_track(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_mixer(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_properties(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_function_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_array(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }
}

//
pub struct PropertyGetter {
    pub assignee: Box<Expression>,
    pub identifier: Identifier,
}

impl<'a> TryFrom<Pair<'a, Rule>> for PropertyGetter {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }
}

//
pub struct MethodCall {
    pub caller: Expression,
    pub callee: Vec<FunctionExpression>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for MethodCall {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }
}

//
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

//
pub enum FunctionExpression {
    Function(FunctionCall),
    FunctionList(Vec<FunctionCall>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for FunctionExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = Ast::inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::FunctionCall => FunctionExpression::from_func_call(inner),
            Rule::FunctionListCall => FunctionExpression::from_func_call_list(inner),
            _ => Ast::parse_rule_error::<Self>(&inner),
        }
    }
}

impl FunctionExpression {
    fn from_func_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(FunctionExpression::Function(pair.try_into()?))
    }

    fn from_func_call_list(pair: Pair<Rule>) -> ParseResult<Self> {
        let functions: ParseResult<Vec<FunctionCall>> =
            pair.into_inner().map(FunctionCall::try_from).collect();
        Ok(FunctionExpression::FunctionList(functions?))
    }
}

//
pub struct FunctionCall {
    pub identifier: Box<Expression>,
    pub parameters: Option<Vec<Expression>>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for FunctionCall {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        dbg!(&pair);
        unimplemented!()
    }
}
