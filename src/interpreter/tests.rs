use super::*;
use crate::parser::{CollyParser, Rule};

#[test]
fn interpret_pattern_complex() {
    let mut context = Context::default();
    // let result: ast::Pattern = tests::parse_source_for_rule("|01 2|", Rule::Pattern).unwrap();

    let result: ast::Pattern = CollyParser::parse_source_for_rule(
        "| 01*:23 01[0 1 23]* (012 34)* 01(23 4)5* 1: |",
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
        CollyParser::parse_source_for_rule("a*._:", Rule::Event).unwrap();
    let octave = Octave::default();
    let event_interpreter = EventInterpreter {
        depth: 0,
        event,
        beat: 0,
        octave: Rc::new(RefCell::new(octave)),
        octave_change: None,
        position: 0.0,
    };

    assert_eq!(
        vec![
            IntermediateEvent {
                value: Audible::Degree(Degree::from(10)),
                duration: 3.0,
                octave: None,
                position: 0.0,
            },
            IntermediateEvent {
                value: Audible::Tie,
                duration: 0.5,
                octave: None,
                position: 3.0,
            }
        ],
        event_interpreter.interpret(&mut context).unwrap()
    );
}
