#[cfg(test)]
mod tests;

use crate::ast;
use crate::clock::{ Clock, CursorPosition };
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
        let mut octave = types::Octave::default();
        for (beat, beat_event) in self.0.into_iter().enumerate() {
            let mut beat_event_interpreter = BeatEventInterpreter {
                depth: 0,
                beat,
                beat_event,
                pattern: &pattern,
                octave: &octave,
            };
            beat_event_interpreter.interpret(context)?;
        }

        Ok(pattern)
    }
}

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
    octave: &'a types::Octave,
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
                octave: &mut self.octave,
                octave_change: None,
                alteration: None,
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
    octave: &'a types::Octave,
    octave_change: Option<types::Octave>,
    alteration: Option<i64>,
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
        mut self,
        atoms: Vec<ast::PatternAtom>,
        context: &Context<'_>,
    ) -> InterpreterResult<()> {
        let event_size = atoms
            .iter()
            .filter(|atom| match atom {
                ast::PatternAtom::EventMethod(_)
                | ast::PatternAtom::Octave(_)
                | ast::PatternAtom::Alteration(_) => false,
                _ => true,
            })
            .count();
        let mut position = CursorPosition::from((self.beat as u64, 0));

        for atom in atoms.into_iter() {
            self.interpret_atom(atom, event_size, &mut position, context)?;
        }

        Ok(())
    }

    #[allow(clippy::unit_arg)]
    fn interpret_atom(
        &mut self,
        atom: ast::PatternAtom,
        event_size: usize,
        position: &mut CursorPosition,
        context: &Context<'_>,
    ) -> InterpreterResult<()> {
        match atom {
            ast::PatternAtom::EventMethod(method) => unimplemented!(),
            ast::PatternAtom::Octave(octave) => {
                Ok(self.interpret_octave_change(octave))
            }
            ast::PatternAtom::Alteration(alteration) => {
                Ok(self.interpret_alteration(alteration))
            }
            ast::PatternAtom::Pitch(pitch) => {
                let degree = self.interpret_pitch(pitch);
                Ok(self.schedule_degree(degree, event_size))
            },
            ast::PatternAtom::Pause => unimplemented!(),
            ast::PatternAtom::MacroTarget => unimplemented!(),
            ast::PatternAtom::Modulation(modulation) => unimplemented!(),
        }
    }

    fn interpret_octave_change(&mut self, octave: ast::Octave) {
        let octave_change =
            self.octave_change.get_or_insert(types::Octave::default());
        match octave {
            ast::Octave::Up => octave_change.up(),
            ast::Octave::Down => octave_change.down(),
        }
    }

    fn interpret_alteration(&mut self, alteration: ast::Alteration) {
        let alteration_change = self.alteration.get_or_insert(0);
        match alteration {
            ast::Alteration::Up => *alteration_change += 1,
            ast::Alteration::Down => *alteration_change -= 1,
        }
    }

    fn interpret_pitch(&mut self, pitch: u64) -> types::Degree {
        let mut degree = types::Degree::from(pitch);
        if let Some(alteration) = self.alteration.take() {
            degree.alteration = alteration;
        }
        
        degree
    }

    fn schedule_degree(&mut self, degree: types::Degree, event_size: usize) {
        if let Some(octave) = self.octave_change.take() {
            //TODO: schedule octave change
        }

        //TODO: schedule the degree
    }

}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error during interpretation of {}: {}", 0, 1)]
    Rule(String, String),
}
