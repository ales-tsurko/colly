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
            Expression::Variable(id) => Ok(context.variables.get(&id.into())),
            Expression::PatternSuperExpression(value) => unimplemented!(),
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
        pattern.sort();

        Ok(pattern)
    }
}

#[derive(Debug, Default)]
struct PatternInnerInterpreter {
    divisor_multiplier: usize,
    octave: Rc<RefCell<types::Octave>>,
    inner: Vec<ast::BeatEvent>,
    interpret_ties: bool,
}

impl PatternInnerInterpreter {
    fn new(inner: Vec<ast::BeatEvent>) -> Self {
        PatternInnerInterpreter {
            inner,
            interpret_ties: true,
            ..Default::default()
        }
    }
}

impl Interpreter<Vec<IntermediateEvent>> for PatternInnerInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut intermediates: Vec<ArrangedIntermediates> = Vec::new();
        for (beat, event) in self.inner.iter().enumerate() {
            let mut events = BeatEventInterpreter {
                beat: beat as u64,
                event: event.clone(),
                octave: self.octave.clone(),
                divisor_multiplier: self.divisor_multiplier,
            }
            .interpret(context)?;

            intermediates.append(&mut events);
        }

        self.interpret_ties_or_concat(intermediates, context)
    }
}

impl PatternInnerInterpreter {
    fn interpret_ties_or_concat(
        &self,
        intermediates: Vec<ArrangedIntermediates>,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        if self.interpret_ties {
            let tie_interpreter = TieInterpreter::new(intermediates);
            tie_interpreter.interpret(context)
        } else {
            Ok(intermediates
                .into_iter()
                .map(|arranged| arranged.values)
                .flatten()
                .collect())
        }
    }
}

#[derive(Debug, Default)]
struct TieInterpreter {
    result: Vec<IntermediateEvent>,
    previous_indices: Vec<usize>,
    intermediates: Vec<ArrangedIntermediates>,
}

impl Interpreter<Vec<IntermediateEvent>> for TieInterpreter {
    fn interpret(
        mut self,
        _context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        if self.intermediates.is_empty() {
            return Ok(Vec::new());
        }

        self.prepare_buffers()?;

        for (beat, mut values) in self
            .intermediates
            .clone()
            .into_iter()
            .map(|arranged| (arranged.beat, arranged.values))
        {
            let mid = self.previous_indices.len().min(values.len());
            let next = values.split_off(mid);

            self.interpret_current(values);
            self.interpret_next(next, beat)?;
        }

        Ok(self.result)
    }
}

impl TieInterpreter {
    fn new(intermediates: Vec<ArrangedIntermediates>) -> Self {
        TieInterpreter {
            intermediates,
            result: Vec::new(),
            previous_indices: Vec::new(),
        }
    }

    fn prepare_buffers(&mut self) -> InterpreterResult<()> {
        self.check_lonely()?;

        self.result = self.intermediates.remove(0).values;
        self.previous_indices =
            self.result.iter().enumerate().map(|(n, _)| n).collect();

        Ok(())
    }

    fn check_lonely(&self) -> InterpreterResult<()> {
        if let Some(arranged) = self.intermediates.get(0) {
            for event in arranged.values.iter() {
                if let Audible::Tie = event.value {
                    return Err(InterpreterError::LonelyTie(
                        self.intermediates[0].beat,
                    ));
                }
            }
        }

        Ok(())
    }

    fn interpret_current(&mut self, values: Vec<IntermediateEvent>) {
        for (n, prev_n) in
            self.previous_indices.split_off(0).into_iter().enumerate()
        {
            let current = values[n % values.len()].clone();
            match current.value {
                Audible::Tie => {
                    self.result[prev_n].duration += current.duration;
                    self.previous_indices.push(prev_n);
                }
                _ => {
                    if values.len() > n {
                        self.push_result(current);
                    }
                }
            }
        }
    }

    fn interpret_next(
        &mut self,
        next: Vec<IntermediateEvent>,
        beat: u64,
    ) -> InterpreterResult<()> {
        for event in next.into_iter() {
            match event.value {
                Audible::Tie => return Err(InterpreterError::LonelyTie(beat)),
                _ => self.push_result(event),
            }
        }

        Ok(())
    }

    fn push_result(&mut self, event: IntermediateEvent) {
        self.result.push(event);
        self.previous_indices.push(self.result.len() - 1);
    }
}

#[derive(Debug, Clone)]
struct BeatEventInterpreter {
    event: ast::BeatEvent,
    beat: u64,
    octave: Rc<RefCell<types::Octave>>,
    divisor_multiplier: usize,
}

impl BeatEventInterpreter {
    fn normalize(
        &self,
        group: Vec<ArrangedIntermediates>,
    ) -> Vec<ArrangedIntermediates> {
        let mut divisor =
            group.iter().fold(0.0, |acc, event| acc + event.duration);
        if self.divisor_multiplier > 0 {
            divisor *= self.divisor_multiplier as f64;
        }

        group
            .into_iter()
            .map(|mut arranged| {
                arranged.values.iter_mut().for_each(|mut event| {
                    event.duration /= divisor;
                    event.beat_position /= divisor;
                });
                arranged
            })
            .map(|mut arranged| {
                arranged.duration /= divisor;
                arranged.beat_position /= divisor;
                arranged
            })
            .collect()
    }
}

impl Interpreter<Vec<ArrangedIntermediates>> for BeatEventInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<ArrangedIntermediates>> {
        let mut output: Vec<ArrangedIntermediates> = Vec::new();
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

        Ok(self.normalize(output))
    }
}

#[derive(Debug, Clone)]
struct EventInterpreter {
    event: ast::Event,
    beat: u64,
    octave: Rc<RefCell<types::Octave>>,
    beat_position: Rc<RefCell<f64>>,
}

impl Interpreter<Vec<ArrangedIntermediates>> for EventInterpreter {
    fn interpret(
        self,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<ArrangedIntermediates>> {
        match self.event.clone() {
            ast::Event::Group(atoms) => self.interpret_group(atoms),
            ast::Event::Chord(chord) => self.interpret_chord(chord, context),
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
    ) -> InterpreterResult<Vec<ArrangedIntermediates>> {
        let mut output: Vec<ArrangedIntermediates> = Vec::new();
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
    ) -> InterpreterResult<Vec<ArrangedIntermediates>> {
        let intermediates =
            self.interpret_inner(event.inner.len(), event.inner, context)?;
        let methods_modifier =
            AtomInterpreter::interpret_methods(1.0, &event.methods);

        Ok(intermediates
            .into_iter()
            .scan(self.beat_position.borrow_mut(), |position, mut event| {
                event.duration *= methods_modifier;
                event.beat_position = **position;
                **position += event.duration;
                event.beat = self.beat;
                Some(ArrangedIntermediates::from(event))
            })
            .collect())
    }

    fn interpret_inner(
        &self,
        num_of_beats: usize,
        inner: Vec<ast::BeatEvent>,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<IntermediateEvent>> {
        let mut inner_interpreter = PatternInnerInterpreter::new(inner);
        inner_interpreter.interpret_ties = false;
        inner_interpreter.divisor_multiplier = num_of_beats;
        inner_interpreter.interpret(context)
    }

    fn interpret_chord(
        self,
        chord: ast::Chord,
        context: &mut Context<'_>,
    ) -> InterpreterResult<Vec<ArrangedIntermediates>> {
        let intermediates = self.interpret_inner(1, chord.inner, context)?;
        let methods_modifier =
            AtomInterpreter::interpret_methods(1.0, &chord.methods);
        let mut position = self.beat_position.borrow_mut();

        let values = intermediates
            .into_iter()
            .map(|mut event| {
                event.duration *= methods_modifier;
                event.beat = self.beat;
                event.beat_position += *position;
                event
            })
            .collect();

        let duration = 1.0 * methods_modifier;
        let result = vec![ArrangedIntermediates {
            values,
            duration,
            beat: self.beat,
            beat_position: *position,
        }];

        *position += duration;

        Ok(result)
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
    ) -> InterpreterResult<Option<ArrangedIntermediates>> {
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
                Ok(Some(self.next_intermediate(Audible::Pause, &atom.methods)))
            }
            ast::PatternAtomValue::PatternInlet => unimplemented!(),
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
    ) -> ArrangedIntermediates {
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

        intermediate.into()
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

#[derive(Clone, Debug, Default, PartialEq)]
struct ArrangedIntermediates {
    values: Vec<IntermediateEvent>,
    duration: f64,
    beat: u64,
    beat_position: f64,
}

impl From<IntermediateEvent> for ArrangedIntermediates {
    fn from(intermediate: IntermediateEvent) -> Self {
        Self {
            duration: intermediate.duration,
            beat: intermediate.beat,
            beat_position: intermediate.beat_position,
            values: vec![intermediate],
        }
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
    #[fail(display = "Alone Tie at beat number {}", 0)]
    LonelyTie(u64),
}
