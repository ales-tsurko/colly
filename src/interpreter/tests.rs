use super::*;
use crate::parser::ast::tests;
use crate::parser::Rule;

#[test]
fn interpret_pattern() {
    let mut context = Context::default();
    // let result: ast::Pattern = tests::parse_source_for_rule("|01 2|", Rule::Pattern).unwrap();

    let result: ast::Pattern = tests::parse_source_for_rule("| 01*:23 * *01[0 1 23]* (012 34)* 01(23 4)5* |", Rule::Pattern).unwrap();
    // let pattern: types::Pattern = result.interpret(&mut context).unwrap();
    // let expected = ...;
    // assert_eq!(expected, result);
    // 0.0 = 1/8 . 1/8 = (1/8 + 1/16) 1/8 = (1/8 + 1/16) (1/8 - 1/16)
}