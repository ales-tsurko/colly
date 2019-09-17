#[cfg(test)]
mod tests;

use crate::{
    ast,
    clock::{Clock, CursorPosition},
    types::{self, Function, Identifier, Mixer, Value},
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
            Expression::Identifier(value) => {
                Ok(Value::from(Identifier::from(value)))
            }
            Expression::Variable(id) => Ok(context.variables.get(&id.into())),
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

impl From<ast::Identifier> for Identifier {
    fn from(id: ast::Identifier) -> Self {
        Self(id.0)
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
        let octave = Rc::new(RefCell::new(types::Octave::default()));
        let mut intermediates: Vec<IntermediateEvent> = Vec::new();
        for (beat, beat_event) in self.0.into_iter().enumerate() {
            intermediates.append(
                &mut BeatEventInterpreter {
                    depth: 0,
                    beat,
                    beat_event,
                    octave: octave.clone(),
                }
                .interpret(context)?,
            );
        }

        let mut pattern =
            types::Pattern::new(context.mixer.clock.cursor().clone());
        // TODO: schedule events

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
struct BeatEventInterpreter {
    depth: usize,
    beat_event: ast::BeatEvent,
    beat: usize,
    octave: Rc<RefCell<types::Octave>>,
}

impl Node for BeatEventInterpreter {
    fn depth(&self) -> usize {
        self.depth
    }

    fn beat(&self) -> usize {
        self.beat
    }
}

impl Interpreter<Vec<IntermediateEvent>> for BeatEventInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut output: Vec<IntermediateEvent> = Vec::new();
        for event in self.clone().beat_event.0 {
            output.append(
                &mut EventInterpreter {
                    depth: self.depth(),
                    event,
                    beat: self.beat(),
                    octave: self.octave.clone(),
                    position: 0.0,
                }
                .interpret(context)?,
            );
        }

        Ok(output)
    }
}

#[derive(Debug, Clone)]
struct EventInterpreter {
    event: ast::Event,
    depth: usize,
    beat: usize,
    octave: Rc<RefCell<types::Octave>>,
    position: f64,
}

impl Node for EventInterpreter {
    fn depth(&self) -> usize {
        self.depth
    }

    fn beat(&self) -> usize {
        self.beat
    }
}

impl Interpreter<Vec<IntermediateEvent>> for EventInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        match self.event.clone() {
            ast::Event::Group(atoms) => self.interpret_group(atoms),
            ast::Event::Chord(chord) => self.interpret_chord(chord),
            ast::Event::ParenthesisedEvent(event) => {
                self.interpret_parenthesised(event)
            }
        }
    }
}

impl EventInterpreter {
    fn interpret_group(
        self,
        atoms: Vec<ast::PatternAtom>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut output: Vec<IntermediateEvent> = Vec::new();
        let mut atom_interpreter = AtomInterpreter::new(
            self.octave.clone(),
            self.beat(),
            self.position,
        );

        for atom in atoms.into_iter() {
            if let Some(intermediate) = atom_interpreter.interpret(atom)? {
                output.push(intermediate);
            }
        }

        Ok(output)
    }

    fn interpret_parenthesised(
        self,
        event: ast::ParenthesisedEvent,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        for event in event.inner {}

        unimplemented!()
    }

    fn interpret_chord(
        self,
        chord: ast::Chord,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
struct AtomInterpreter {
    octave: Rc<RefCell<types::Octave>>,
    octave_change: Option<types::Octave>,
    position: f64,
    beat: usize,
}

impl AtomInterpreter {
    fn new(
        octave: Rc<RefCell<types::Octave>>,
        beat: usize,
        position: f64,
    ) -> Self {
        Self {
            octave,
            beat,
            position,
            ..Default::default()
        }
    }

    #[allow(clippy::unit_arg)]
    fn interpret(
        &mut self,
        atom: ast::PatternAtom,
    ) -> InterpreterResult<Option<IntermediateEvent>> {
        match atom.value {
            ast::PatternAtomValue::Octave(octave) => {
                self.interpret_octave_change(octave);
                Ok(None)
            }
            ast::PatternAtomValue::Tie => {
                Ok(Some(self.next_intermediate(Audible::Tie, &atom.methods)))
            }
            ast::PatternAtomValue::Note(note) => {
                let value = Audible::Degree(self.interpret_note(note));
                Ok(Some(self.next_intermediate(value, &atom.methods)))
            }
            ast::PatternAtomValue::Pause => {
                let value = Audible::Pause;
                Ok(Some(self.next_intermediate(value, &atom.methods)))
            }
            ast::PatternAtomValue::MacroTarget => unimplemented!(),
            ast::PatternAtomValue::Modulation(modulation) => unimplemented!(),
        }
    }

    fn interpret_octave_change(&mut self, octave: ast::Octave) {
        let mut global_octave = self.octave.borrow_mut();
        let octave_change =
            self.octave_change.get_or_insert(global_octave.clone());
        match octave {
            ast::Octave::Up => {
                octave_change.up();
                global_octave.up();
            }
            ast::Octave::Down => {
                octave_change.down();
                global_octave.down();
            }
        }
    }

    fn next_intermediate(
        &mut self,
        value: Audible,
        methods: &[ast::EventMethod],
    ) -> IntermediateEvent {
        let duration = AtomInterpreter::interpret_methods(1.0, methods);
        let intermediate = IntermediateEvent {
            value,
            duration,
            octave: self.octave_change.take(),
            position: self.position,
            beat: self.beat,
        };
        self.position += duration;

        intermediate
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

    fn interpret_methods(source: f64, methods: &[ast::EventMethod]) -> f64 {
        methods
            .iter()
            .fold(source, |duration, method| match method {
                ast::EventMethod::Multiply => duration * 2.0,
                ast::EventMethod::Divide => duration / 2.0,
                ast::EventMethod::Dot => duration * 1.5,
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IntermediateEvent {
    value: Audible,
    octave: Option<types::Octave>,
    duration: f64,
    position: f64,
    beat: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Audible {
    Degree(types::Degree),
    Modulation(types::Modulation),
    Pause,
    Tie,
}

impl IntermediateEvent {
    fn schedule(mut self, pattern: &mut types::Pattern) {
        if let Some(octave) = self.octave.take() {
            //TODO:
        }

        match self.value {
            Audible::Degree(degree) => unimplemented!(),
            Audible::Modulation(modulation) => unimplemented!(),
            Audible::Pause => (),
            Audible::Tie => unreachable!(),
        }
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error during interpretation of {}: {}", 0, 1)]
    Rule(String, String),
}
