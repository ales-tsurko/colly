#[cfg(test)]
mod tests;

use crate::ast;
use crate::clock::Clock;
use crate::types::{self, Function, Identifier, Mixer, Value};
use std::collections::HashMap;
use std::rc::Rc;

type InterpreterResult<T> = Result<T, InterpreterError>;

pub trait Interpreter<V> {
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<V>;
}

pub struct Context<'a> {
    parent: &'a Option<Context<'a>>,
    mixer: Mixer,
    variables: VariablesTable,
    functions: HashMap<Identifier, Box<dyn Function<Item = Value>>>,
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
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<()> {
        for statement in self.0.into_iter() {
            statement.interpret(context)?;
        }
        Ok(())
    }
}

impl Interpreter<()> for ast::Statement {
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<()> {
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
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<Value> {
        match self {
            ast::SuperExpression::Expression(value) => value.interpret(context),
            ast::SuperExpression::Method(value) => value.interpret(context),
        }
    }
}

impl Interpreter<Value> for ast::Expression {
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<Value> {
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
        context: &mut Context<'_>,
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
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<Value> {
        unimplemented!()
    }
}

impl Interpreter<()> for ast::Assignment {
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<()> {
        unimplemented!()
    }
}

impl Interpreter<types::Pattern> for ast::Pattern {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<types::Pattern> {
        let mut pattern =
            types::Pattern::new(context.mixer.clock.cursor().clone());
        for (beat, beat_event) in self.0.into_iter().enumerate() {
            let mut beat_event_interpreter = BeatEventInterpreter {
                depth: 0,
                beat,
                beat_event,
                pattern: &pattern,
            };
            beat_event_interpreter.interpret(context)?;
        }

        Ok(pattern)
    }
}

//
trait Node {
    fn beat(&self) -> usize;

    fn depth(&self) -> usize;

    fn duration_divisor(&self) -> usize {
        self.depth().pow(2)
    }
}

#[derive(Debug, Clone)]
struct BeatEventInterpreter<'a> {
    depth: usize,
    beat_event: ast::BeatEvent,
    beat: usize,
    pattern: &'a types::Pattern,
}

impl<'a> Node for BeatEventInterpreter<'a> {
    fn depth(&self) -> usize {
        self.depth
    }

    fn beat(&self) -> usize {
        self.beat
    }
}

impl<'a> Interpreter<()> for BeatEventInterpreter<'a> {
    fn interpret(mut self, context: &mut Context<'_>) -> InterpreterResult<()> {
        for (n, event) in self.clone().beat_event.0.into_iter().enumerate() {
                EventInterpreter {
                    depth: self.depth(),
                    event,
                    beat: n,
                    pattern: &mut self.pattern,
                }
                .interpret(context)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct EventInterpreter<'a> {
    depth: usize,
    event: ast::Event,
    beat: usize,
    pattern: &'a types::Pattern,
}

impl<'a> Node for EventInterpreter<'a> {
    fn depth(&self) -> usize {
        self.depth
    }

    fn beat(&self) -> usize {
        self.beat
    }
}

impl<'a> Interpreter<()> for EventInterpreter<'a> {
    fn interpret(self, context: &mut Context<'_>) -> InterpreterResult<()> {
        match self.clone().event {
            ast::Event::Group(atoms) => self.interpret_group(atoms, context),
            ast::Event::Chord(event_groups) => unimplemented!(),
            ast::Event::ParenthesisedEvent(event_groups) => unimplemented!(),
            ast::Event::EventMethod(event_method) => unimplemented!(),
        }
    }
}

impl<'a> EventInterpreter<'a> {
    fn interpret_group(
        self,
        atoms: Vec<ast::PatternAtom>,
        context: &Context<'_>,
    ) -> InterpreterResult<()> {
        unimplemented!()
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error during interpretation of {}: {}", 0, 1)]
    Rule(String, String),
}
