use crate::parser::Rule;
use crate::parser_error_from_string_with_pair;
use pest::{ 
    iterators::{Pair, Pairs},
    error::{Error, ErrorVariant},
};
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Ast<'a>(pub Vec<Statement<'a>>);

impl<'a, 'b: 'a> TryFrom<Pairs<'a, Rule>> for Ast<'b> {
    type Error = Error<Rule>;

    fn try_from(pairs: Pairs<Rule>) -> Result<Self, Self::Error> {
        dbg!(pairs);
        //TODO parse pairs here
        //let mut nodes: Vec<Statement> = vec![];
        //for pair in pairs {
        //let statement = Statement::try_from(&rule)?;
        //nodes.push(statement);
        //}
        //Ast(nodes);
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
        let pairs = CollyParser::parse(Rule::file, "\n(hello world)\n(world hello)\n")
            .unwrap();
        let ast = Ast::try_from(pairs).unwrap();
    }
}

pub enum Statement<'a> {
    Assign(Assign<'a>),
    Method,
    Function(FunctionExpression<'a>),
}

pub enum Assign<'a> {
    Pattern(Expression<'a>, PatternSuperExpression<'a>),
    Variable {
        assignee: Identifier<'a>,
        assignment: SuperExpression<'a>,
    },
    Properties {
        assignee: SuperExpression<'a>,
        assignment: Expression<'a>
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
        identifier: Identifier<'a>
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
    Expression(PatternExpression<'a>)
}

pub struct PatternExpression<'a> { 
    pub pattern: Pattern,
    pub inner_method: Option<FunctionExpression<'a>>,
    pub methods: Option<Vec<FunctionExpression<'a>>>,
    pub properties: Option<Properties<'a>>,
}

pub struct Pattern {
    pub inner: Vec<Event>
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
