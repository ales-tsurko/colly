//! Abstract syntax tree types.
// Below you'll see patterns like
// inner.next().unwrap()
// These unwraps are fine, because we relate on the parser's rules,
// which were checked on the parsing phase.

#[cfg(test)]
pub(crate) mod tests;

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

    fn try_from(pairs: Pairs<'_, Rule>) -> ParseResult<Self> {
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
    Assign(Assignment),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::SuperExpression => Statement::from_super_expression(inner),
            Rule::AssignStatement => Statement::from_assign(inner),
            _ => CollyParser::rule_error(&inner),
        }
    }
}

impl Statement {
    fn from_super_expression(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        Ok(Statement::SuperExpression(pair.try_into()?))
    }

    fn from_assign(pair: Pair<'_, Rule>) -> ParseResult<Self> {
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

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::Expression => SuperExpression::from_expression(inner),
            Rule::MethodCall => SuperExpression::from_method_call(inner),
            _ => CollyParser::rule_error(&inner),
        }
    }
}

impl SuperExpression {
    fn from_expression(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Expression(pair.try_into()?))
    }

    fn from_method_call(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Method(pair.try_into()?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    PropertyGetter {
        assignee: Box<Expression>,
        property_id: Vec<Identifier>,
    },
    Boolean(bool),
    Variable(Identifier),
    PatternSuperExpression(PatternSuperExpression),
    Number(f64),
    String(String),
    PatternSlot((u64, u64)),
    Track(u64),
    Mixer,
    Properties(Properties),
    Array(Vec<SuperExpression>),
    Function(FunctionCall),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Expression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Expression::from_variant(inner)
    }
}

impl Expression {
    pub fn from_variant(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::PropertyGetter => Expression::from_property_getter(pair),
            Rule::Boolean => Expression::from_boolean(pair),
            Rule::FunctionCall => Expression::from_function_call(pair),
            Rule::Variable => Expression::from_variable(pair),
            Rule::PatternSuperExpression => {
                Expression::from_pattern_super_expression(pair)
            }
            Rule::Number => Expression::from_number(pair),
            Rule::String => Expression::from_string(pair),
            Rule::PatternSlot => Expression::from_pattern_slot(pair),
            Rule::Track => Expression::from_track(pair),
            Rule::Mixer => Ok(Expression::Mixer),
            Rule::Properties => Expression::from_properties(pair),
            Rule::Array => Expression::from_array(pair),
            _ => CollyParser::rule_error(&pair),
        }
    }

    fn from_property_getter(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let assignee: Box<Expression> =
            Box::new(Expression::from_variant(inner.next().unwrap())?);
        let ids: ParseResult<Vec<Identifier>> =
            inner.map(Identifier::try_from).collect();
        Ok(Expression::PropertyGetter {
            assignee,
            property_id: ids?,
        })
    }

    fn from_boolean(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        CollyParser::assert_rule(Rule::Boolean, &pair)?;
        let value: bool = pair.as_str().parse().unwrap();
        Ok(Expression::Boolean(value))
    }

    fn from_variable(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Ok(Expression::Variable(inner.try_into()?))
    }

    fn from_pattern_super_expression(
        pair: Pair<'_, Rule>,
    ) -> ParseResult<Self> {
        Ok(Expression::PatternSuperExpression(pair.try_into()?))
    }

    fn from_number(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let number: f64 = pair.as_str().parse().unwrap();
        Ok(Expression::Number(number))
    }

    fn from_string(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        let value: String = inner.as_str().parse().unwrap();
        Ok(Expression::String(value))
    }

    fn from_pattern_slot(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let error = CollyParser::error("Cannot parse pattern slot.", &pair);
        let mut inner = pair.into_inner();
        if let Expression::Track(track) =
            Expression::from_track(inner.next().unwrap())?
        {
            let slot_number: u64 =
                inner.next().unwrap().as_str().parse().unwrap();
            return Ok(Expression::PatternSlot((track, slot_number)));
        }
        Err(error)
    }

    fn from_track(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let _ = inner.next().unwrap();
        let track_number: u64 = inner.next().unwrap().as_str().parse().unwrap();
        Ok(Expression::Track(track_number))
    }

    fn from_properties(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        Ok(Expression::Properties(pair.try_into()?))
    }

    fn from_function_call(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        Ok(Expression::Function(pair.try_into()?))
    }

    fn from_array(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let superexpressions: ParseResult<Vec<SuperExpression>> =
            pair.into_inner().map(SuperExpression::try_from).collect();
        Ok(Expression::Array(superexpressions?))
    }
}

//
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Identifier(pub String);

impl<'a> TryFrom<Pair<'a, Rule>> for Identifier {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        CollyParser::assert_rule(Rule::Identifier, &pair)?;
        Ok(Identifier(pair.as_str().to_string()))
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

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let identifier: ParseResult<Identifier> =
            inner.next().unwrap().try_into();
        let params: ParseResult<Vec<Expression>> =
            inner.map(Expression::try_from).collect();
        Ok(FunctionCall {
            identifier: identifier?,
            parameters: params?,
        })
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub struct Properties(HashMap<Identifier, PropertyValue>);

impl<'a> TryFrom<Pair<'a, Rule>> for Properties {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let map: ParseResult<HashMap<Identifier, PropertyValue>> =
            inner.map(Properties::parse_kv_pair).collect();
        Ok(Properties(map?))
    }
}

impl Properties {
    fn parse_kv_pair(
        pair: Pair<'_, Rule>,
    ) -> ParseResult<(Identifier, PropertyValue)> {
        let mut inner = pair.into_inner();
        let identifier: ParseResult<Identifier> =
            inner.next().unwrap().try_into();
        let value: ParseResult<PropertyValue> =
            inner.next().unwrap().try_into();

        Ok((identifier?, value?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    SuperExpression(SuperExpression),
    PatternExpression(PatternExpression),
}

impl<'a> TryFrom<Pair<'a, Rule>> for PropertyValue {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::SuperExpression => {
                Ok(PropertyValue::SuperExpression(inner.try_into()?))
            }
            Rule::PatternExpression => {
                Ok(PropertyValue::PatternExpression(inner.try_into()?))
            }
            _ => CollyParser::rule_error(&inner),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub struct MethodCall {
    pub caller: Expression,
    pub callee: Vec<FunctionCall>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for MethodCall {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let caller: ParseResult<Expression> = inner.next().unwrap().try_into();
        let callee: ParseResult<Vec<FunctionCall>> =
            inner.map(FunctionCall::try_from).collect();
        Ok(MethodCall {
            caller: caller?,
            callee: callee?,
        })
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Assignment {
    Pattern {
        assignee: Expression,
        assignment: PatternSuperExpression,
    },
    Variable {
        assignee: Identifier,
        assignment: SuperExpression,
    },
    Properties {
        assignee: SuperExpression,
        assignment: Properties,
    },
}

impl<'a> TryFrom<Pair<'a, Rule>> for Assignment {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::PatternAssignment => {
                Assignment::from_pattern_assignment(inner)
            }
            Rule::VariableAssignment => {
                Assignment::from_variable_assignment(inner)
            }
            Rule::PropertiesAssignment => {
                Assignment::form_properties_assignment(inner)
            }
            _ => CollyParser::rule_error(&inner),
        }
    }
}

impl Assignment {
    fn from_variable_assignment(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let error =
            CollyParser::error("Cannot parse variable assignment.", &pair);
        let mut inner = pair.into_inner();
        let variable: ParseResult<Expression> =
            Expression::from_variant(inner.next().unwrap());
        if let Expression::Variable(assignee) = variable? {
            let assignment: SuperExpression =
                inner.next().unwrap().try_into()?;
            return Ok(Assignment::Variable {
                assignee,
                assignment,
            });
        }
        Err(error)
    }

    fn from_pattern_assignment(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let assignee = Expression::from_variant(inner.next().unwrap())?;
        let assignment: PatternSuperExpression =
            inner.next().unwrap().try_into()?;
        Ok(Assignment::Pattern {
            assignee,
            assignment,
        })
    }

    fn form_properties_assignment(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let assignee: SuperExpression = inner.next().unwrap().try_into()?;
        let assignment: Properties = inner.next().unwrap().try_into()?;
        Ok(Assignment::Properties {
            assignee,
            assignment,
        })
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSuperExpression {
    ExpressionList(Vec<PatternExpression>),
    Expression(PatternExpression),
}

impl<'a> TryFrom<Pair<'a, Rule>> for PatternSuperExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::PatternExpression => Self::from_pattern_expression(inner),
            Rule::PatternExpressionList => {
                Self::from_pattern_expression_list(inner)
            }
            _ => CollyParser::rule_error(&inner),
        }
    }
}

impl PatternSuperExpression {
    fn from_pattern_expression(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Ok(PatternSuperExpression::Expression(inner.try_into()?))
    }

    fn from_pattern_expression_list(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let expressions: ParseResult<Vec<PatternExpression>> =
            inner.map(PatternExpression::try_from).collect();
        Ok(PatternSuperExpression::ExpressionList(expressions?))
    }
}

//
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatternExpression {
    pub pattern: Pattern,
    pub methods: Vec<FunctionCall>,
    pub properties: Option<Properties>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for PatternExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();

        let mut result = Self::default();

        for pair in inner {
            match pair.as_rule() {
                Rule::Pattern => result.parse_pattern(pair)?,
                Rule::PatternMethod => result.parse_method(pair)?,
                Rule::Properties => result.parse_properties(pair)?,
                _ => CollyParser::rule_error(&pair)?,
            }
        }

        Ok(result)
    }
}

impl PatternExpression {
    fn parse_pattern(&mut self, pair: Pair<'_, Rule>) -> ParseResult<()> {
        self.pattern = pair.try_into()?;
        Ok(())
    }

    fn parse_method(&mut self, pair: Pair<'_, Rule>) -> ParseResult<()> {
        for next in pair.into_inner() {
            self.methods.push(next.try_into()?);
        }
        Ok(())
    }

    fn parse_properties(&mut self, pair: Pair<'_, Rule>) -> ParseResult<()> {
        self.properties = Some(pair.try_into()?);
        Ok(())
    }
}

//
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pattern(pub Vec<BeatEvent>);

impl<'a> TryFrom<Pair<'a, Rule>> for Pattern {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let events: ParseResult<Vec<BeatEvent>> =
            inner.map(BeatEvent::try_from).collect();
        Ok(Pattern(events?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub struct BeatEvent(pub Vec<Event>);

impl<'a> TryFrom<Pair<'a, Rule>> for BeatEvent {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let events: ParseResult<Vec<Event>> =
            inner.map(Event::try_from).collect();
        Ok(BeatEvent(events?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Group(Vec<PatternAtom>),
    Chord(Chord),
    ParenthesisedEvent(ParenthesisedEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub inner: Vec<BeatEvent>,
    pub methods: Vec<EventMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParenthesisedEvent {
    pub inner: Vec<BeatEvent>,
    pub methods: Vec<EventMethod>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Event {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::Chord => Self::from_chord(inner),
            Rule::Group => Self::from_group(inner),
            Rule::ParenthesisedEventGroup => {
                Self::from_parenthesised_event(inner)
            }
            _ => CollyParser::rule_error(&inner),
        }
    }
}

impl Event {
    fn from_group(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let atoms: ParseResult<Vec<PatternAtom>> =
            inner.map(PatternAtom::try_from).collect();
        Ok(Event::Group(atoms?))
    }

    fn from_chord(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();

        let mut groups: Vec<BeatEvent> = Vec::new();
        let mut methods: Vec<EventMethod> = Vec::new();
        for pair in inner {
            match pair.as_rule() {
                Rule::BeatEvent => groups.push(pair.try_into()?),
                Rule::EventMethod => methods.push(pair.try_into()?),
                _ => CollyParser::rule_error(&pair)?,
            }
        }

        Ok(Event::Chord(Chord {
            inner: groups,
            methods,
        }))
    }

    fn from_parenthesised_event(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();

        let mut groups: Vec<BeatEvent> = Vec::new();
        let mut methods: Vec<EventMethod> = Vec::new();
        for pair in inner {
            match pair.as_rule() {
                Rule::BeatEvent => groups.push(pair.try_into()?),
                Rule::EventMethod => methods.push(pair.try_into()?),
                _ => CollyParser::rule_error(&pair)?,
            }
        }

        Ok(Event::ParenthesisedEvent(ParenthesisedEvent {
            inner: groups,
            methods,
        }))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub struct PatternAtom {
    pub value: PatternAtomValue,
    pub methods: Vec<EventMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternAtomValue {
    Octave(Octave),
    Note(Note),
    Tie,
    Pause,
    PatternInlet(Expression),
    Interpolation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub pitch: u64,
    pub alteration: Vec<Alteration>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for PatternAtom {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let value = inner.next().unwrap();

        match value.as_rule() {
            Rule::Octave => Ok(PatternAtom {
                value: PatternAtomValue::Octave(value.try_into()?),
                methods: Vec::new(),
            }),
            Rule::Note => Ok(PatternAtom {
                value: PatternAtomValue::from_note(value)?,
                methods: Self::parse_methods(inner)?,
            }),
            Rule::Tie => Ok(PatternAtom {
                value: PatternAtomValue::Tie,
                methods: Self::parse_methods(inner)?,
            }),
            Rule::Pause => Ok(PatternAtom {
                value: PatternAtomValue::Pause,
                methods: Self::parse_methods(inner)?,
            }),
            Rule::PatternInlet => Self::from_inlet(value, inner),
            Rule::Interpolation => Ok(PatternAtom {
                value: PatternAtomValue::Interpolation,
                methods: Self::parse_methods(inner)?,
            }),
            _ => CollyParser::rule_error(&value),
        }
    }
}

impl PatternAtom {
    fn parse_methods(pairs: Pairs<'_, Rule>) -> ParseResult<Vec<EventMethod>> {
        let mut methods: Vec<EventMethod> = Vec::new();

        for pair in pairs {
            methods.push(EventMethod::try_from(pair)?);
        }

        Ok(methods)
    }

    fn from_inlet(
        value: Pair<'_, Rule>,
        methods: Pairs<'_, Rule>,
    ) -> ParseResult<Self> {
        let expression = CollyParser::first_inner_for_pair(value)?;

        Ok(PatternAtom {
            value: PatternAtomValue::PatternInlet(expression.try_into()?),
            methods: Self::parse_methods(methods)?,
        })
    }
}

impl PatternAtomValue {
    fn from_note(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let mut alteration: Vec<Alteration> = Vec::new();
        let mut pitch: u64 = 0;

        for pair in inner {
            match pair.as_rule() {
                Rule::Alteration => {
                    alteration.push(Alteration::try_from(pair)?)
                }
                Rule::Pitch => pitch = Self::parse_pitch(pair)?,
                _ => CollyParser::rule_error(&pair)?,
            }
        }

        Ok(Self::Note(Note { pitch, alteration }))
    }

    fn parse_pitch(pair: Pair<'_, Rule>) -> ParseResult<u64> {
        match u64::from_str_radix(pair.as_str(), 16) {
            Ok(value) => Ok(value),
            Err(e) => Err(CollyParser::error(&format!("{:?}", e), &pair)),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum EventMethod {
    Dot,
    Multiply,
    Divide,
}

impl<'a> TryFrom<Pair<'a, Rule>> for EventMethod {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "." => Ok(EventMethod::Dot),
            "*" => Ok(EventMethod::Multiply),
            ":" => Ok(EventMethod::Divide),
            _ => CollyParser::rule_error(&pair),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Octave {
    Up,
    Down,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Octave {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "o" => Ok(Octave::Down),
            "O" => Ok(Octave::Up),
            _ => CollyParser::rule_error(&pair),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Alteration {
    Up,
    Down,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Alteration {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<'_, Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "+" => Ok(Alteration::Up),
            "-" => Ok(Alteration::Down),
            _ => CollyParser::rule_error(&pair),
        }
    }
}
