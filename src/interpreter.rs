use crate::parser::ast::*;
use crate::primitives::{Function, Identifier, Mixer, ValueWrapper, Value};
use std::collections::HashMap;

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
            // Pattern(Pattern),
            Expression::Number(value) => Ok(ValueWrapper::from(value)),
            Expression::String(value) => Ok(ValueWrapper::from(value)),
            // PatternSlot((u64, u64)),
            Expression::Track(index) => Ok(self.interpret_track(index as usize, context)),
            // Mixer,
            // Properties(Properties),
            // Array(Vec<SuperExpression>),
            // Function(FunctionExpression),
            _ => unimplemented!(),
        }
    }
}

impl Expression {
    fn interpret_track(&self, index: usize, context: &mut Context) -> ValueWrapper {
        match context.mixer.clone_track(index) {
            Some(track) => ValueWrapper::from(track),
            None => ValueWrapper::Nothing
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

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error interpret AST.")]
    InterpretAst,
}
