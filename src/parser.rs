
#[derive(Parser)]
#[grammar = "parser/colly.pest"]
pub struct CollyParser;

pub mod ast;
