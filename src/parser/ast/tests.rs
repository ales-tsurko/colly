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
fn test_parse_boolean() {
    let ast: Ast = "true".parse().unwrap();
    let expected = expected_from_boolean(true);
    assert_eq!(ast.0, expected);

    let ast: Ast = "false".parse().unwrap();
    let expected = expected_from_boolean(false);
    assert_eq!(ast.0, expected);

    fn expected_from_boolean(value: bool) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            Expression::Boolean(value),
        ))]
    }
}

#[test]
fn test_parse_string() {
    let source = "\"hello world {} \n\"";
    let ast: Ast = source.parse().unwrap();
    let expected = expected_from_string("hello world {} \n".to_string());
    assert_eq!(ast.0, expected);

    fn expected_from_string(value: String) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            Expression::String(value),
        ))]
    }
}

#[test]
fn test_parse_mixer() {
    let ast: Ast = "$".parse().unwrap();
    let expected = vec![Statement::SuperExpression(SuperExpression::Expression(
        Expression::Mixer,
    ))];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_identifier() {
    let ast: Ast = "foo".parse().unwrap();
    let expected = vec![Statement::SuperExpression(SuperExpression::Expression(
        Expression::Identifier(Identifier("foo".into())),
    ))];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_variable() {
    let ast: Ast = ":foo".parse().unwrap();
    let expected = vec![Statement::SuperExpression(SuperExpression::Expression(
        Expression::Variable(Identifier("foo".into())),
    ))];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_property_getter() {
    let ast: Ast = ":foo.bar".parse().unwrap();
    let expected = expected_from_expr(Expression::PropertyGetter {
        assignee: Box::new(Expression::Variable(Identifier("foo".into()))),
        property_id: Identifier("bar".into()),
    });
    assert_eq!(ast.0, expected);

    fn expected_from_expr(value: Expression) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            value,
        ))]
    }
}
