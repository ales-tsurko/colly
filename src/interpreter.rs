#[cfg(test)]
mod tests;

use crate::{
    ast,
    clock::{Clock, CursorPosition},
    types::{self, Function, Identifier, Mixer, Value},
};

use std::{collections::HashMap, convert::TryFrom, rc::Rc};

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
        (self.depth() + 1).pow(2)
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
            let intermediate = EventInterpreter {
                depth: self.depth(),
                event,
                beat: n,
                pattern: &mut self.pattern,
                octave: &mut self.octave,
                octave_change: None,
            }
            .interpret(context)?;

            dbg!(&intermediate);
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
}

#[derive(Debug, Clone, PartialEq)]
struct IntermediateEvent {
    value: Audible,
    octave: Option<types::Octave>,
    duration: f64,
}

impl IntermediateEvent {
    fn schedule(mut self, pattern: &mut types::Pattern) {
        if let Some(octave) = self.octave.take() {
            //TODO:
        }

        match self.value {
            Audible::Degree(degree) => unimplemented!(),
            Audible::Modulation(modulation) => unimplemented!(),
            Audible::Pause => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Audible {
    Degree(types::Degree),
    Modulation(types::Modulation),
    Pause,
}

impl<'a> Node for EventInterpreter<'a> {
    fn depth(&self) -> usize {
        self.depth
    }

    fn beat(&self) -> usize {
        self.beat
    }
}

impl<'a> Interpreter<Vec<IntermediateEvent>> for EventInterpreter<'a> {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        match self.event.clone() {
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
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        // let event_size = self.group_size(&atoms);
        // let initial_duration = 1.0 / (event_size as f64);
        // let mut group_position = CursorPosition::from((self.beat as u64, 0));

        let mut output: Vec<IntermediateEvent> = Vec::new();

        for atom in atoms.into_iter() {
            self.interpret_atom(atom, &mut output)?;
        }

        Ok(output)
    }

    fn group_size(&self, group: &[ast::PatternAtom]) -> usize {
        group
            .iter()
            .filter(|atom| match atom.value {
                ast::PatternAtomValue::Octave(_) => false,
                _ => true,
            })
            .count()
    }

    #[allow(clippy::unit_arg)]
    fn interpret_atom(
        &mut self,
        atom: ast::PatternAtom,
        output: &mut Vec<IntermediateEvent>,
    ) -> InterpreterResult<()> {
        match atom.value {
            ast::PatternAtomValue::Octave(octave) => {
                Ok(self.interpret_octave_change(octave))
            }
            ast::PatternAtomValue::Tie => unimplemented!(),
            ast::PatternAtomValue::Note(note) => {
                Ok(output.push(IntermediateEvent {
                    value: Audible::Degree(self.interpret_note(note)),
                    duration: self.interpret_methods(&atom.methods),
                    octave: self.octave_change.take(),
                }))
            }
            ast::PatternAtomValue::Pause => {
                Ok(output.push(IntermediateEvent {
                    value: Audible::Pause,
                    duration: self.interpret_methods(&atom.methods),
                    octave: self.octave_change.take(),
                }))
            }
            ast::PatternAtomValue::MacroTarget => unimplemented!(),
            ast::PatternAtomValue::Modulation(modulation) => unimplemented!(),
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

    fn interpret_note(&mut self, note: ast::Note) -> types::Degree {
        let mut degree = types::Degree::from(note.pitch);
        degree.alteration = self.interpret_alteration(note.alteration);

        degree
    }

    fn interpret_alteration(
        &mut self,
        alterations: Vec<ast::Alteration>,
    ) -> i64 {
        alterations
            .into_iter()
            .fold(0, |acc, alteration| match alteration {
                ast::Alteration::Up => acc + 1,
                ast::Alteration::Down => acc - 1,
            })
    }

    fn interpret_methods(&self, methods: &[ast::EventMethod]) -> f64 {
        let mut duration = 1.0 / (self.duration_divisor() as f64);

        for method in methods.iter() {
            match method {
                ast::EventMethod::Multiply => duration *= 2.0,
                ast::EventMethod::Divide => duration /= 2.0,
                ast::EventMethod::Dot => duration *= 1.5,
            }
        }

        duration
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error during interpretation of {}: {}", 0, 1)]
    Rule(String, String),
}
