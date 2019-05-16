use crate::parser::ast::*;
use crate::primitives::{Function, Identifier, Mixer, ValueWrapper};
use std::collections::HashMap;
use failure_derive::Fail;

pub trait Interpreter {
    fn interpret(&self, context: &mut Context) -> Result<(), InterpreterError>;
}

pub struct Context {
    mixer: Mixer,
    variables_table: HashMap<Identifier, SuperExpression>,
    functions_table: HashMap<Identifier, Box<Function<Item = ValueWrapper>>>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            mixer: Mixer::default(),
            variables_table: HashMap::new(),
            functions_table: HashMap::new(),
        }
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error interpret AST.")]
    InterpretAst,
}    