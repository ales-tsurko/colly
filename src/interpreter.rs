use crate::parser::ast::*;
use crate::types::{Function, Identifier, Mixer};
use std::collections::HashMap;
use failure_derive::Fail;

type InterpreterResult<T> = Result<T, InterpreterError>;

pub struct Interpreter {
    mixer: Mixer,
    variables_table: HashMap<Identifier, SuperExpression>,
    functions_table: HashMap<Identifier, Box<Function>>,
}

impl<'a> Default for Interpreter {
    fn default() -> Self {
        Interpreter {
            mixer: Mixer::default(),
            variables_table: HashMap::new(),
            functions_table: HashMap::new(),
        }
    }
}

impl Interpreter {
    pub fn inperpret_ast(ast: &Ast) -> InterpreterResult<()> {
        unimplemented!()
    }
}

#[derive(Debug, Fail)]
pub enum InterpreterError {
    #[fail(display = "Error interpret AST.")]
    InterpretAst,
}    