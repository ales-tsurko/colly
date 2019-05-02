use std::collections::HashMap;

pub enum Ast<'a> {
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
