#[cfg(test)]
mod tests;

use crate::clock::Clock;
use crate::parser::ast;
use crate::types::{
    self, Function, Identifier, Mixer, Value,
};
use std::collections::HashMap;
use std::rc::Rc;

type InterpreterResult<T> = Result<T, InterpreterError>;

pub trait Interpreter<V> {
    fn interpret(self, context: &mut Context)
        -> InterpreterResult<V>;
}

pub struct Context<'a> {
    parent: &'a Option<Context<'a>>,
    mixer: Mixer,
    variables: VariablesTable,
    functions: HashMap<Identifier, Box<Function<Item = Value>>>,
}

#[derive(Debug, Default)]
pub struct VariablesTable(pub HashMap<Identifier, Value>);

impl VariablesTable {
    fn get(&self, id: &Identifier) -> Value {
        match self.0.get(id) {
            Some(value) => value.clone(),
            None => Value::Nothing,
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

impl Interpreter<()> for ast::Ast {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<()> {
        for statement in self.0.into_iter() {
            statement.interpret(context)?;
        }
        Ok(())
    }
}

impl Interpreter<()> for ast::Statement {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<()> {
        match self {
            ast::Statement::SuperExpression(value) => {
                let _ = value.interpret(context)?;
                Ok(())
            }
            ast::Statement::Assign(value) => value.interpret(context),
        }
    }
}

impl Interpreter<Value> for ast::SuperExpression {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Value> {
        match self {
            ast::SuperExpression::Expression(value) => value.interpret(context),
            ast::SuperExpression::Method(value) => value.interpret(context),
        }
    }
}

impl Interpreter<Value> for ast::Expression {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Value> {
            use ast::Expression;
        match self {
            Expression::Boolean(value) => Ok(Value::from(value)),
            Expression::Identifier(value) => Ok(Value::from(value)),
            Expression::Variable(id) => Ok(context.variables.get(&id)),
            Expression::Pattern(value) => {
                Ok(Value::from(value.interpret(context)?))
            }
            Expression::Number(value) => Ok(Value::from(value)),
            Expression::String(value) => Ok(Value::from(value)),
            Expression::PatternSlot((track_n, slot_n)) => {
                Expression::interpret_slot(
                    track_n as usize,
                    slot_n as usize,
                    context,
                )
            }
            Expression::Track(index) => {
                Ok(Value::from(context.mixer.track(index as usize)))
            }
            Expression::Mixer => Ok(Value::Mixer),
            // Properties(Properties),
            // Array(Vec<SuperExpression>),
            // Function(FunctionExpression),
            _ => unimplemented!(),
        }
    }
}

impl ast::Expression {
    fn interpret_slot(
        track_n: usize,
        slot_n: usize,
        context: &mut Context,
    ) -> InterpreterResult<Value> {
        match Rc::get_mut(&mut context.mixer.track(track_n)) {
            Some(track) => Ok(Value::from(track.slot(slot_n))),
            None => Err(InterpreterError::Rule(
                "expression".into(),
                "Cannot get track reference".into(),
            )),
        }
    }
}

impl Interpreter<Value> for ast::MethodCall {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Value> {
        unimplemented!()
    }
}

impl Interpreter<()> for ast::Assignment {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<()> {
        unimplemented!()
    }
}

impl Interpreter<types::Pattern> for ast::Pattern {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<types::Pattern> {
        let mut events: Vec<types::Event> = Vec::new();
        let beat_length = context.mixer.clock.beat_length();
        for (n, group) in self.0.into_iter().enumerate() {
            events.append(
                &mut BeatEventNode {
                    level: 0,
                    event_group: group,
                    start_position: beat_length * (n as u64),
                }
                .interpret(context)?,
            );
        }

        Ok(types::Pattern {
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
struct BeatEventNode {
    level: u64,
    event_group: ast::BeatEvent,
    start_position: u64,
}

impl Node for BeatEventNode {
    fn level(&self) -> u64 {
        self.level
    }

    fn start_position(&self) -> u64 {
        self.start_position
    }
}

impl Interpreter<Vec<types::Event>> for BeatEventNode {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Vec<types::Event>> {
        let mut events: Vec<types::Event> = Vec::new();
        let beat_length = self.beat_length(&context.mixer.clock);
        for (n, event) in self.clone().event_group.0.into_iter().enumerate() {
            events.append(
                &mut EventNode {
                    level: self.level(),
                    event,
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
    event: ast::Event,
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

impl Interpreter<Vec<types::Event>> for EventNode {
    fn interpret(
        self,
        context: &mut Context,
    ) -> InterpreterResult<Vec<types::Event>> {
        match self.clone().event {
            ast::Event::Group(atoms) => self.interpret_group(atoms, context),
            ast::Event::Chord(event_groups) => unimplemented!(),
            ast::Event::ParenthesisedEvent(event_groups) => unimplemented!(),
            ast::Event::EventMethod(event_method) => unimplemented!(),
        }
    }
}

impl EventNode {
    fn interpret_group(
        self,
        atoms: Vec<ast::PatternAtom>,
        context: &Context,
    ) -> InterpreterResult<Vec<types::Event>> {
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