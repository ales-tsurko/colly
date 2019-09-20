#[cfg(test)]
mod tests;

use crate::{
    ast,
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
        let inner_interpreter = PatternInnerInterpreter::new(self.0);
        let intermediates = inner_interpreter.interpret(context)?;

        let mut pattern =
            types::Pattern::new(context.mixer.clock.cursor().clone());
        // TODO: schedule events

        Ok(pattern)
    }
}

#[derive(Debug, Default)]
struct PatternInnerInterpreter {
    divisor_multiplier: usize,
    octave: Rc<RefCell<types::Octave>>,
    inner: Vec<ast::BeatEvent>,
}

impl PatternInnerInterpreter {
    fn new(inner: Vec<ast::BeatEvent>) -> Self {
        PatternInnerInterpreter {
            inner,
            ..Default::default()
        }
    }

    fn normalize_intermediates(
        &self,
        intermediates: Vec<IntermediateEvent>,
    ) -> Vec<IntermediateEvent> {
        let mut divisor = intermediates
            .iter()
            .fold(0.0, |acc, event| acc + event.duration);
        if self.divisor_multiplier > 0 {
            divisor *= self.divisor_multiplier as f64;
        }

        intermediates
            .into_iter()
            .map(|mut event| {
                event.duration /= divisor;
                event.beat_position /= divisor;
                event
            })
            .collect()
    }

    fn handle_ties(
        &self,
        intermediates: Vec<IntermediateEvent>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        if !intermediates.is_empty() && intermediates[0].value == Audible::Tie {
            return Err(InterpreterError::LonelyTie);
        }

        Ok(intermediates
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, event)| event.value != Audible::Tie)
            .map(|(mut n, mut event)| {
                if intermediates.is_empty() {
                    return event;
                }

                while let Some(next_event) = intermediates.get(n + 1) {
                    match next_event.value {
                        Audible::Tie => event.duration += next_event.duration,
                        _ => break,
                    }
                    n += 1;
                }

                event
            })
            .collect())
    }
}

impl Interpreter<Vec<IntermediateEvent>> for PatternInnerInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut intermediates: Vec<IntermediateEvent> = Vec::new();
        for (beat, event) in self.inner.iter().enumerate() {
            let events = BeatEventInterpreter {
                beat: beat as u64,
                event: event.clone(),
                octave: self.octave.clone(),
            }
            .interpret(context)?;

            intermediates.append(&mut self.normalize_intermediates(events));
        }

        self.handle_ties(intermediates)
    }
}

#[derive(Debug, Clone)]
struct BeatEventInterpreter {
    event: ast::BeatEvent,
    beat: u64,
    octave: Rc<RefCell<types::Octave>>,
}

impl Interpreter<Vec<IntermediateEvent>> for BeatEventInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut output: Vec<IntermediateEvent> = Vec::new();
        let beat_position = Rc::new(RefCell::new(0.0));
        for event in self.clone().event.0 {
            output.append(
                &mut EventInterpreter {
                    event,
                    beat: self.beat,
                    octave: self.octave.clone(),
                    beat_position: beat_position.clone(),
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
    beat: u64,
    octave: Rc<RefCell<types::Octave>>,
    beat_position: Rc<RefCell<f64>>,
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
                self.interpret_parenthesised(event, context)
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
            self.beat,
            self.beat_position.clone(),
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
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let num_of_beats = event.inner.len();
        let mut inner_interpreter = PatternInnerInterpreter::new(event.inner);
        inner_interpreter.divisor_multiplier = num_of_beats;
        let intermediates = inner_interpreter.interpret(context)?;

        let methods_modifier =
            AtomInterpreter::interpret_methods(1.0, &event.methods);
        let mut beat_position = self.beat_position.borrow_mut();

        Ok(intermediates
            .into_iter()
            .map(|mut event| {
                event.duration *= methods_modifier;
                event.beat_position = *beat_position;
                *beat_position += event.duration;
                event.beat = self.beat;
                event
            })
            .collect())
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
    position: Rc<RefCell<f64>>,
    beat: u64,
}

impl AtomInterpreter {
    fn new(
        octave: Rc<RefCell<types::Octave>>,
        beat: u64,
        position: Rc<RefCell<f64>>,
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
        let mut beat_position = self.position.borrow_mut();
        let intermediate = IntermediateEvent {
            value,
            duration,
            octave: self.octave_change.take(),
            beat_position: *beat_position,
            beat: self.beat,
        };
        *beat_position += duration;

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

#[derive(Clone, Debug, PartialEq)]
struct IntermediateEvent {
    value: Audible,
    octave: Option<types::Octave>,
    duration: f64,
    beat_position: f64,
    beat: u64,
}

#[derive(Clone, Debug, PartialEq)]
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
            // TODO: there must haven't been any ties at this stage,
            // because they should be handled in the Pattern interpreter
            // but there is a place for a human mistake, so let's
            // think: how to do it better?
            Audible::Tie => unreachable!(),
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum InterpreterError {
    #[fail(display = "Error during interpretation of {}: {}", 0, 1)]
    Rule(String, String),
    #[fail(display = "A tie must have a root event, which it prolongs.")]
    LonelyTie,
}
