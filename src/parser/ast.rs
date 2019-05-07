#[cfg(test)]
mod tests;
use crate::parser::Rule;
use crate::parser::{CollyParser, ParseResult};
use pest::Parser;
use pest::{
    error::Error,
    iterators::{Pair, Pairs},
};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Ast(pub Vec<Statement>);

impl<'a> TryFrom<Pairs<'a, Rule>> for Ast {
    type Error = Error<Rule>;

    fn try_from(pairs: Pairs<Rule>) -> ParseResult<Self> {
        let statements: ParseResult<Vec<Statement>> = pairs
            .filter(|pair| pair.as_rule() == Rule::Statement)
            .map(Statement::try_from)
            .collect();
        Ok(Ast(statements?))
    }
}

impl FromStr for Ast {
    type Err = Error<Rule>;

    fn from_str(source: &str) -> ParseResult<Self> {
        let pairs = CollyParser::parse(Rule::File, source)?;
        Ast::try_from(pairs)
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    SuperExpression(SuperExpression),
    Assign(Assign),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::SuperExpression => Statement::from_super_expression(inner),
            Rule::AssignStatement => Statement::from_assign(inner),
            _ => CollyParser::rule_error::<Self>(&inner),
        }
    }
}

impl Statement {
    fn from_super_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::SuperExpression(pair.try_into()?))
    }

    fn from_assign(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Statement::Assign(pair.try_into()?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum SuperExpression {
    Expression(Expression),
    Method(MethodCall),
}

impl<'a> TryFrom<Pair<'a, Rule>> for SuperExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::Expression => SuperExpression::from_expression(inner),
            Rule::MethodCall => SuperExpression::from_method_call(inner),
            _ => CollyParser::rule_error::<Self>(&inner),
        }
    }
}

impl SuperExpression {
    pub fn from_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Expression(pair.try_into()?))
    }

    pub fn from_method_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Method(pair.try_into()?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    PropertyGetter {
        assignee: Box<Expression>,
        property_id: Identifier,
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

impl<'a> TryFrom<Pair<'a, Rule>> for Expression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Expression::from_variant(inner)
    }
}

impl Expression {
    pub fn from_variant(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::PropertyGetter => Expression::from_property_getter(pair),
            Rule::Boolean => Expression::from_boolean(pair),
            Rule::FunctionExpression => {
                Expression::from_function_expression(pair)
            }
            Rule::Identifier => Expression::from_identifier(pair),
            Rule::Variable => Expression::from_variable(pair),
            Rule::PatternString => Expression::from_pattern_string(pair),
            Rule::Number => Expression::from_number(pair),
            Rule::String => Expression::from_string(pair),
            Rule::PatternSlot => Expression::from_pattern_slot(pair),
            Rule::Track => Expression::from_track(pair),
            Rule::Mixer => Ok(Expression::Mixer),
            Rule::Properties => Expression::from_properties(pair),
            Rule::Array => Expression::from_array(pair),
            _ => CollyParser::rule_error::<Self>(&pair),
        }
    }

    pub fn from_property_getter(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.clone().into_inner();
        let assignee: Box<Expression> = Box::new(Expression::from_variant(
            CollyParser::next_pair(&mut inner, &pair)?,
        )?);
        let raw_identifier = CollyParser::next_pair(&mut inner, &pair)?;
        Ok(Expression::PropertyGetter {
            assignee,
            property_id: raw_identifier.into(),
        })
    }

    pub fn from_boolean(pair: Pair<Rule>) -> ParseResult<Self> {
        let value: bool = pair.as_str().parse().unwrap();
        Ok(Expression::Boolean(value))
    }

    pub fn from_identifier(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Expression::Identifier(pair.into()))
    }

    pub fn from_variable(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Ok(Expression::Variable(inner.into()))
    }

    pub fn from_pattern_string(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_number(pair: Pair<Rule>) -> ParseResult<Self> {
        let number: f64 = pair.as_str().parse().unwrap();
        Ok(Expression::Number(number))
    }

    pub fn from_string(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        let value: String = inner.as_str().parse().unwrap();
        Ok(Expression::String(value))
    }

    pub fn from_pattern_slot(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.clone().into_inner();
        if let Expression::Track(track) =
            Expression::from_track(CollyParser::next_pair(&mut inner, &pair)?)?
        {
            let slot_number: u64 = CollyParser::next_pair(&mut inner, &pair)?
                .as_str()
                .parse()
                .unwrap();
            return Ok(Expression::PatternSlot((track, slot_number)));
        }
        CollyParser::rule_error(&pair)
    }

    pub fn from_track(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.clone().into_inner();
        let _ = CollyParser::next_pair(&mut inner, &pair)?;
        let track_number: u64 = CollyParser::next_pair(&mut inner, &pair)?
            .as_str()
            .parse()
            .unwrap();
        Ok(Expression::Track(track_number))
    }

    pub fn from_properties(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    pub fn from_function_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Expression::Function(pair.try_into()?))
    }

    pub fn from_array(pair: Pair<Rule>) -> ParseResult<Self> {
        let superexpressions: ParseResult<Vec<SuperExpression>> =
            pair.into_inner().map(SuperExpression::try_from).collect();
        Ok(Expression::Array(superexpressions?))
    }
}

//
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Identifier(pub String);

impl<'a> From<Pair<'a, Rule>> for Identifier {
    fn from(pair: Pair<Rule>) -> Self {
        Identifier(pair.as_str().to_string())
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionExpression {
    Function(FunctionCall),
    FunctionList(Vec<FunctionCall>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for FunctionExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::FunctionCall => FunctionExpression::from_func_call(inner),
            Rule::FunctionListCall => {
                FunctionExpression::from_func_call_list(inner)
            }
            _ => CollyParser::rule_error::<Self>(&inner),
        }
    }
}

impl FunctionExpression {
    fn from_func_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(FunctionExpression::Function(pair.try_into()?))
    }

    fn from_func_call_list(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.clone().into_inner();
        let unparsed_funcs: ParseResult<Vec<Pair<Rule>>> =
            inner.map(CollyParser::first_inner_for_pair).collect();
        let func_calls: ParseResult<Vec<FunctionCall>> = unparsed_funcs?
            .into_iter()
            .map(FunctionCall::try_from)
            .collect();
        Ok(FunctionExpression::FunctionList(func_calls?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub identifier: Identifier,
    pub parameters: Vec<Expression>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for FunctionCall {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.clone().into_inner();
        let identifier: Identifier =
            CollyParser::next_pair(&mut inner, &pair)?.into();
        let params: ParseResult<Vec<Expression>> =
            inner.map(Expression::try_from).collect();
        Ok(FunctionCall {
            identifier,
            parameters: params?,
        })
    }
}

//
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Properties(HashMap<Key, Value>);

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Key(Identifier);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    SuperExpression(SuperExpression),
    PatternExpression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternSuperExpression {
    ExpressionList(Vec<PatternExpression>),
    Expression(PatternExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternExpression {
    pub pattern: Pattern,
    pub inner_method: Option<FunctionExpression>,
    pub methods: Option<Vec<FunctionExpression>>,
    pub properties: Option<Properties>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub inner: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Chord(Vec<Event>),
    Group(Vec<PatternSymbol>),
    ParenthesisedEvent(Vec<Event>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternSymbol {
    EventMethod(EventMethod),
    Octave(Octave),
    Alteration(Alteration),
    Pitch(u64),
    Pause,
    PatternInput,
    Modulation(Modulation),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventMethod {
    Tie,
    Dot,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Octave {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Alteration {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modulation {
    Down,
    Up,
    Crescendo,
    Diminuendo,
    Literal(f64),
}
