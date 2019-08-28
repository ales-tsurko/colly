use super::*;
use crate::parser::{CollyParser, Rule};

#[test]
fn interpret_pattern_complex() {
    let mut context = Context::default();
    // let result: ast::Pattern = tests::parse_source_for_rule("|01 2|", Rule::Pattern).unwrap();

    let result: ast::Pattern = CollyParser::parse_source_for_rule(
        "| 01*:23 * *01[0 1 23]* (012 34)* 01(23 4)5* 1: |",
        Rule::Pattern,
    )
    .unwrap();
    // let pattern: types::Pattern = result.interpret(&mut context).unwrap();
    // let expected = ...;
    // assert_eq!(expected, result);
}

#[test]
#[ignore]
fn interpret_pattern_simple() {
    use types::*;

    let mut context = Context::default();
    let ast_result: ast::Pattern =
        CollyParser::parse_source_for_rule("| 0 0 0 |", Rule::Pattern).unwrap();
    let result: Pattern = ast_result.interpret(&mut context).unwrap();
    // let expected = Pattern {
    //     stream: EventStream::from(vec![
    //         (EventType::Pitch(0.0), EventState::On, 0.into()).into(),
    //         (EventType::Pitch(0.0), EventState::Off, 1.into()).into(),
    //         (EventType::Pitch(0.0), EventState::On, 1.into()).into(),
    //         (EventType::Pitch(0.0), EventState::Off, 2.into()).into(),
    //         (EventType::Pitch(0.0), EventState::On, 2.into()).into(),
    //         (EventType::Pitch(0.0), EventState::Off, 3.into()).into(),
    //     ]),
    // };

    // assert_eq!(expected, result);
}
