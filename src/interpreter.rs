use crate::clock::Clock;
use crate::parser::ast::*;
use crate::primitives::{
    self, Function, Identifier, Mixer, Value, ValueWrapper,
};
use std::collections::HashMap;
use std::rc::Rc;

type InterpreterResult<T> = Result<T, InterpreterError>;

pub trait Interpreter {
    type Value;
    fn interpret(self, context: &mut Context)
        -> InterpreterResult<Self::Value>;
}

pub struct Context<'a> {
    parent: &'a Option<Context<'a>>,
    mixer: Mixer,
    variables: VariablesTable,
    functions: HashMap<Identifier, Box<Function<Item = ValueWrapper>>>,
}

#[derive(Debug, Default)]
pub struct VariablesTable(pub HashMap<Identifier, ValueWrapper>);

impl VariablesTable {
    fn get(&self, id: &Identifier) -> ValueWrapper {
        match self.0.get(id) {
            Some(value) => value.clone(),
            None => ValueWrapper::Nothing,
        }
    }
}

impl<'a> Default for Context<'a> {
    fn default() -> Self {
        Context {
            parent: &None,
            mixer: Mixer::default(),
            variables: VariablesTable::default(),
            functions: HashMap::default(),
        }
    }
}

impl Interpreter for Ast {
    type Value = ();

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        for statement in self.0.into_iter() {
            statement.interpret(context)?;
        }
        Ok(())
    }
}

impl Interpreter for Statement {
    type Value = ();

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        match self {
            Statement::SuperExpression(value) => {
                let _ = value.interpret(context)?;
                Ok(())
            }
            Statement::Assign(value) => value.interpret(context),
        }
    }
}

impl Interpreter for SuperExpression {
    type Value = ValueWrapper;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        match self {
            SuperExpression::Expression(value) => value.interpret(context),
            SuperExpression::Method(value) => value.interpret(context),
        }
    }
}

impl Interpreter for Expression {
    type Value = ValueWrapper;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        match self {
            Expression::Boolean(value) => Ok(ValueWrapper::from(value)),
            Expression::Identifier(value) => Ok(ValueWrapper::from(value)),
            Expression::Variable(id) => Ok(context.variables.get(&id)),
            Expression::Pattern(value) => {
                Ok(ValueWrapper::from(value.interpret(context)?))
            }
            Expression::Number(value) => Ok(ValueWrapper::from(value)),
            Expression::String(value) => Ok(ValueWrapper::from(value)),
            Expression::PatternSlot((track_n, slot_n)) => {
                Expression::interpret_slot(
                    track_n as usize,
                    slot_n as usize,
                    context,
                )
            }
            Expression::Track(index) => {
                Ok(ValueWrapper::from(context.mixer.track(index as usize)))
            }
            Expression::Mixer => Ok(ValueWrapper::Mixer),
            // Properties(Properties),
            // Array(Vec<SuperExpression>),
            // Function(FunctionExpression),
            _ => unimplemented!(),
        }
    }
}

impl Expression {
    fn interpret_slot(
        track_n: usize,
        slot_n: usize,
        context: &mut Context,
    ) -> InterpreterResult<ValueWrapper> {
        match Rc::get_mut(&mut context.mixer.track(track_n)) {
            Some(track) => Ok(ValueWrapper::from(track.slot(slot_n))),
            None => Err(InterpreterError::Rule(
                "expression".into(),
                "Cannot get track reference".into(),
            )),
        }
    }
}

impl Interpreter for MethodCall {
    type Value = ValueWrapper;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        unimplemented!()
    }
}

impl Interpreter for Assignment {
    type Value = ();

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        unimplemented!()
    }
}

impl Interpreter for Pattern {
    type Value = primitives::Pattern;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        let mut events: Vec<primitives::Event> = Vec::new();
        let beat_length = context.mixer.clock.beat_length();
        for (n, group) in self.0.into_iter().enumerate() {
            events.append(
                &mut EventGroupNode {
                    level: 0,
                    event_group: group,
                    start_position: beat_length * (n as u64),
                }
                .interpret(context)?,
            );
        }

        Ok(primitives::Pattern {
            stream: events.into(),
        })
    }
}

//
trait Node {
    fn start_position(&self) -> u64;

    fn level(&self) -> u64;

    fn beat_divisor(&self) -> u64 {
        self.level().pow(2)
    }

    fn beat_length(&self, clock: &Clock) -> u64 {
        clock.beat_length() / self.beat_divisor()
    }
}

#[derive(Debug, Clone)]
struct EventGroupNode {
    level: u64,
    event_group: EventGroup,
    start_position: u64,
}

impl Node for EventGroupNode {
    fn level(&self) -> u64 {
        self.level
    }

    fn start_position(&self) -> u64 {
        self.start_position
    }
}

impl Interpreter for EventGroupNode {
    type Value = Vec<primitives::Event>;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        let mut events: Vec<primitives::Event> = Vec::new();
        let beat_length = self.beat_length(&context.mixer.clock);
        for (n, event) in self.clone().event_group.0.into_iter().enumerate() {
            events.append(
                &mut EventNode {
                    level: self.level(),
                    event: event,
                    start_position: beat_length * (n as u64),
                }
                .interpret(context)?,
            );
        }

        Ok(events)
    }
}

#[derive(Debug, Clone)]
struct EventNode {
    level: u64,
    event: Event,
    start_position: u64,
}

impl Node for EventNode {
    fn level(&self) -> u64 {
        self.level
    }

    fn start_position(&self) -> u64 {
        self.start_position
    }
}

impl Interpreter for EventNode {
    type Value = Vec<primitives::Event>;

    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Self::Value> {
        match self.clone().event {
            Event::Group(atoms) => self.interpret_group(atoms, context),
            Event::Chord(event_groups) => unimplemented!(),
            Event::ParenthesisedEvent(event_groups) => unimplemented!(),
            Event::EventMethod(event_method) => unimplemented!(),
        }
    }
}

impl EventNode {
    fn interpret_group(
        self,
        atoms: Vec<PatternAtom>,
        context: &Context,
    ) -> InterpreterResult<Vec<primitives::Event>> {
        let beat_length = self.beat_length(&context.mixer.clock);
        let event_length = beat_length / (atoms.len() as u64);
        for (n, atom) in atoms.into_iter().enumerate() {
            let start_position = self.start_position() + (event_length * (n as u64));
            //TODO interpret atom
        }

        unimplemented!()
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error interpret {}: {}", 0, 1)]
    Rule(String, String),
}
