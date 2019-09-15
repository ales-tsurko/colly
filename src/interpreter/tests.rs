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
fn interpret_event_group_pitch() {
    use types::*;

    let mut context = Context::default();
    let event: ast::Event =
        CollyParser::parse_source_for_rule("a*.0:", Rule::Event).unwrap();
    let pattern = Pattern::default();
    let octave = Octave::default();
    let event_interpreter = EventInterpreter {
        depth: 0,
        event,
        beat: 0,
        pattern: &pattern,
        octave: &octave,
        octave_change: None,
        alteration: None,
    };

    assert_eq!(
        vec![
            IntermediateEvent {
                value: Audible::Degree(Degree::from(10)),
                duration: 3.0,
                octave: None,
                alteration: None,
            },
            IntermediateEvent {
                value: Audible::Degree(Degree::default()),
                duration: 0.5,
                octave: None,
                alteration: None,
            }
        ],
        event_interpreter.interpret(&mut context).unwrap()
    );
}
