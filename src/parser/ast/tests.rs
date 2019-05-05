use super::*;
use crate::parser::CollyParser;
use pest::Parser;

#[test]
fn test_parse_property_getter() {
    let pairs = CollyParser::parse(Rule::File, "\n:foo.bar\n").unwrap();
    let _ast = Ast::try_from(pairs).unwrap();
}
