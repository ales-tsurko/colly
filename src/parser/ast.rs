use std::collections::HashMap;

pub enum Ast<'a> {
    Assign(Assign<'a>),
    Method,
    Function(FunctionExpression<'a>),
}

pub enum Assign<'a> {
    Pattern,
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
    PropertyGetter,
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
