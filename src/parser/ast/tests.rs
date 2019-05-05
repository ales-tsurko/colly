use super::*;

#[test]
fn test_parse_number() {
    let ast: Ast = "0.123".parse().unwrap();
    let expected = expected_from_float(0.123);
    assert_eq!(ast.0, expected);

    let ast: Ast = "10".parse().unwrap();
    let expected = expected_from_float(10.0);
    assert_eq!(ast.0, expected);

    let ast: Ast = "1e5".parse().unwrap();
    let expected = expected_from_float(1e5);
    assert_eq!(ast.0, expected);

    let ast: Ast = "1e-5".parse().unwrap();
    let expected = expected_from_float(1e-5);
    assert_eq!(ast.0, expected);

    fn expected_from_float(number: f64) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            Expression::Number(number),
        ))]
    }
}

#[test]
fn test_parse_property_getter() {
    let pairs = CollyParser::parse(Rule::File, "\n:foo.bar\n").unwrap();
    let _ast = Ast::try_from(pairs).unwrap();
}
