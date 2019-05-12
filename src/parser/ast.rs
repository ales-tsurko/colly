// Below you'll see patterns like
// inner.next().unwrap()
// These unwraps are fine, because we relate on the parser rules,
// which were checked on the parsing phase.

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
    Assign(Assignment),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Statement {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::SuperExpression => Statement::from_super_expression(inner),
            Rule::AssignStatement => Statement::from_assign(inner),
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }
}

impl SuperExpression {
    fn from_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(SuperExpression::Expression(pair.try_into()?))
    }

    fn from_method_call(pair: Pair<Rule>) -> ParseResult<Self> {
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
    Identifier(Identifier),
    Variable(Identifier),
    Pattern,
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
            Rule::Pattern => Expression::from_pattern(pair),
            Rule::Number => Expression::from_number(pair),
            Rule::String => Expression::from_string(pair),
            Rule::PatternSlot => Expression::from_pattern_slot(pair),
            Rule::Track => Expression::from_track(pair),
            Rule::Mixer => Ok(Expression::Mixer),
            Rule::Properties => Expression::from_properties(pair),
            Rule::Array => Expression::from_array(pair),
            _ => unreachable!(),
        }
    }

    fn from_property_getter(pair: Pair<Rule>) -> ParseResult<Self> {
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

    fn from_boolean(pair: Pair<Rule>) -> ParseResult<Self> {
        CollyParser::assert_rule(Rule::Boolean, &pair)?;
        let value: bool = pair.as_str().parse().unwrap();
        Ok(Expression::Boolean(value))
    }

    fn from_identifier(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Expression::Identifier(pair.try_into()?))
    }

    fn from_variable(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Ok(Expression::Variable(inner.try_into()?))
    }

    fn from_pattern(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    fn from_number(pair: Pair<Rule>) -> ParseResult<Self> {
        let number: f64 = pair.as_str().parse().unwrap();
        Ok(Expression::Number(number))
    }

    fn from_string(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        let value: String = inner.as_str().parse().unwrap();
        Ok(Expression::String(value))
    }

    fn from_pattern_slot(pair: Pair<Rule>) -> ParseResult<Self> {
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

    fn from_track(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let _ = inner.next().unwrap();
        let track_number: u64 = inner.next().unwrap().as_str().parse().unwrap();
        Ok(Expression::Track(track_number))
    }

    fn from_properties(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Expression::Properties(pair.try_into()?))
    }

    fn from_function_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(Expression::Function(pair.try_into()?))
    }

    fn from_array(pair: Pair<Rule>) -> ParseResult<Self> {
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        CollyParser::assert_rule(Rule::Identifier, &pair)?;
        Ok(Identifier(pair.as_str().to_string()))
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
            _ => unreachable!(),
        }
    }
}

impl FunctionExpression {
    fn from_func_call(pair: Pair<Rule>) -> ParseResult<Self> {
        Ok(FunctionExpression::Function(pair.try_into()?))
    }

    fn from_func_call_list(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let map: ParseResult<HashMap<Identifier, PropertyValue>> =
            inner.map(Properties::parse_kv_pair).collect();
        Ok(Properties(map?))
    }
}

impl Properties {
    fn parse_kv_pair(
        pair: Pair<Rule>,
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
    PatternExpression,
}

impl<'a> TryFrom<Pair<'a, Rule>> for PropertyValue {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::SuperExpression => {
                Ok(PropertyValue::SuperExpression(inner.try_into()?))
            }
            Rule::PatternExpression => unimplemented!(),
            _ => unreachable!(),
        }
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
        let mut inner = pair.into_inner();
        let caller: ParseResult<Expression> = inner.next().unwrap().try_into();
        let callee: ParseResult<Vec<FunctionExpression>> =
            inner.map(FunctionExpression::try_from).collect();
        Ok(MethodCall {
            caller: caller?,
            callee: callee?,
        })
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Assignment {
    Pattern(Expression, PatternSuperExpression),
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::PatternAssignment => {
                Assignment::from_pattern_assignment(pair)
            }
            Rule::VariableAssignment => {
                Assignment::from_variable_assignment(pair)
            }
            Rule::PropertiesAssignment => {
                Assignment::form_properties_assignment(pair)
            }
            _ => unreachable!(),
        }
    }
}

impl Assignment {
    fn from_variable_assignment(pair: Pair<Rule>) -> ParseResult<Self> {
        let error =
            CollyParser::error("Cannot parse variable assignment.", &pair);
        let mut inner = pair.into_inner();
        let variable: ParseResult<Expression> =
            Expression::from_variant(inner.next().unwrap());
        if let Expression::Variable(identifier) = variable? {
            let assignment: ParseResult<SuperExpression> =
                inner.next().unwrap().try_into();
            return Ok(Assignment::Variable {
                assignee: identifier,
                assignment: assignment?,
            });
        }
        Err(error)
    }

    fn from_pattern_assignment(pair: Pair<Rule>) -> ParseResult<Self> {
        unimplemented!()
    }

    fn form_properties_assignment(pair: Pair<Rule>) -> ParseResult<Self> {
        let mut inner = pair.into_inner();
        let assignee: ParseResult<SuperExpression> =
            inner.next().unwrap().try_into();
        let assignment: ParseResult<Properties> =
            inner.next().unwrap().try_into();
        Ok(Assignment::Properties {
            assignee: assignee?,
            assignment: assignment?,
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::PatternExpression => Self::from_pattern_expression(inner),
            Rule::PatternExpressionList => {
                Self::from_pattern_expression_list(inner)
            }
            _ => unreachable!(),
        }
    }
}

impl PatternSuperExpression {
    fn from_pattern_expression(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        Ok(PatternSuperExpression::Expression(inner.try_into()?))
    }

    fn from_pattern_expression_list(pair: Pair<Rule>) -> ParseResult<Self> {
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
    pub inner_method: Option<FunctionExpression>,
    pub methods: Vec<FunctionExpression>,
    pub properties: Option<Properties>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for PatternExpression {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();

        let mut result = Self::default();

        for pair in inner {
            match pair.as_rule() {
                Rule::Pattern => result.parse_pattern(pair)?,
                Rule::PatternInnerMethod => result.parse_inner_method(pair)?,
                Rule::FunctionExpression => result.parse_method(pair)?,
                Rule::Properties => result.parse_properties(pair)?,
                _ => unreachable!(),
            }
        }

        Ok(result)
    }
}

impl PatternExpression {
    fn parse_pattern(&mut self, pair: Pair<Rule>) -> ParseResult<()> {
        self.pattern = pair.try_into()?;
        Ok(())
    }

    fn parse_inner_method(&mut self, pair: Pair<Rule>) -> ParseResult<()> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        self.inner_method = Some(inner.try_into()?);
        Ok(())
    }

    fn parse_method(&mut self, pair: Pair<Rule>) -> ParseResult<()> {
        self.methods.push(pair.try_into()?);
        Ok(())
    }

    fn parse_properties(&mut self, pair: Pair<Rule>) -> ParseResult<()> {
        self.properties = Some(pair.try_into()?);
        Ok(())
    }
}

//
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pattern(pub Vec<Event>);

impl<'a> TryFrom<Pair<'a, Rule>> for Pattern {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let events: ParseResult<Vec<Event>> =
            inner.map(Event::try_from).collect();
        Ok(Pattern(events?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Chord(Vec<Event>),
    Group(Vec<PatternAtom>),
    ParenthesisedEvent(Vec<Event>),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Event {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_rule() {
            Rule::Chord => Self::from_chord(pair),
            Rule::Group => Self::from_group(pair),
            Rule::ParenthesisedEvent => Self::from_parenthesised_event(pair),
            _ => unreachable!(),
        }
    }
}

impl Event {
    fn from_chord(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let events: ParseResult<Vec<Event>> =
            inner.map(Self::try_from).collect();
        Ok(Event::Chord(events?))
    }

    fn from_group(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let atoms: ParseResult<Vec<PatternAtom>> =
            inner.map(PatternAtom::try_from).collect();
        Ok(Event::Group(atoms?))
    }

    fn from_parenthesised_event(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = pair.into_inner();
        let events: ParseResult<Vec<Event>> =
            inner.map(Self::try_from).collect();
        Ok(Event::ParenthesisedEvent(events?))
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum PatternAtom {
    EventMethod(EventMethod),
    Octave(Octave),
    Alteration(Alteration),
    Pitch(u64),
    Pause,
    PatternInput,
    Modulation(Modulation),
}

impl<'a> TryFrom<Pair<'a, Rule>> for PatternAtom {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;

        match inner.as_rule() {
            Rule::EventMethod => {
                Ok(PatternAtom::EventMethod(inner.try_into()?))
            }
            Rule::Octave => Ok(PatternAtom::Octave(inner.try_into()?)),
            Rule::Alteration => Ok(PatternAtom::Alteration(inner.try_into()?)),
            Rule::Pitch => Self::from_pitch(inner),
            Rule::Pause => Ok(PatternAtom::Pause),
            Rule::PatternInput => Ok(PatternAtom::PatternInput),
            Rule::Modulation => Ok(PatternAtom::Modulation(inner.try_into()?)),
            _ => unreachable!(),
        }
    }
}

impl PatternAtom {
    fn from_pitch(pair: Pair<Rule>) -> ParseResult<Self> {
        match u64::from_str_radix(pair.as_str(), 16) {
            Ok(value) => Ok(PatternAtom::Pitch(value)),
            Err(e) => Err(CollyParser::error(&format!("{:?}", e), &pair)),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum EventMethod {
    Tie,
    Dot,
    Multiply,
    Divide,
}

impl<'a> TryFrom<Pair<'a, Rule>> for EventMethod {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "_" => Ok(EventMethod::Tie),
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
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

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "+" => Ok(Alteration::Up),
            "-" => Ok(Alteration::Down),
            _ => CollyParser::rule_error(&pair),
        }
    }
}

//
#[derive(Debug, Clone, PartialEq)]
pub enum Modulation {
    Down,
    Up,
    Crescendo,
    Diminuendo,
    Literal(f64),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Modulation {
    type Error = Error<Rule>;

    fn try_from(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        match inner.as_rule() {
            Rule::ModulationSymbol => Self::from_symbol(inner),
            Rule::LiteralModulation => Self::from_literal_modulation(inner),
            _ => unreachable!()
        }
    }
}

impl Modulation {
    fn from_literal_modulation(pair: Pair<Rule>) -> ParseResult<Self> {
        let inner = CollyParser::first_inner_for_pair(pair)?;
        let value: f64 = inner.as_str().parse().unwrap();
        Ok(Modulation::Literal(value))
    }

    fn from_symbol(pair: Pair<Rule>) -> ParseResult<Self> {
        match pair.as_str() {
            "p" => Ok(Modulation::Down),
            "F" => Ok(Modulation::Up),
            "<" => Ok(Modulation::Diminuendo),
            ">" => Ok(Modulation::Crescendo),
            _ => CollyParser::rule_error(&pair),
        }
    }
}
