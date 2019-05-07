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
    let expected = vec![Statement::SuperExpression(
        SuperExpression::Expression(Expression::Mixer),
    )];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_variable() {
    let ast: Ast = ":foo".parse().unwrap();
    let expected =
        vec![Statement::SuperExpression(SuperExpression::Expression(
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

    //TODO
    // let ast: Ast = ":foo.bar.baz.fred".parse().unwrap();

    fn expected_from_expr(value: Expression) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            value,
        ))]
    }
}

#[test]
fn test_parse_track() {
    let ast: Ast = "$0".parse().unwrap();
    let expected = vec![Statement::SuperExpression(
        SuperExpression::Expression(Expression::Track(0)),
    )];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_patttern_slot() {
    let ast: Ast = "$0.1".parse().unwrap();
    let expected = vec![Statement::SuperExpression(
        SuperExpression::Expression(Expression::PatternSlot((0, 1))),
    )];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_function_expression() {
    let ast: Ast = "foo".parse().unwrap();
    let expected = expected_from_func_call(FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: Vec::new(),
    });
    assert_eq!(ast.0, expected);

    let ast: Ast = "(foo bar)".parse().unwrap();
    let expected = expected_from_func_call(FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![expression_from_func_call(FunctionCall {
            identifier: Identifier("bar".to_string()),
            parameters: Vec::new(),
        })],
    });
    assert_eq!(ast.0, expected);

    let ast: Ast = "(foo true)".parse().unwrap();
    let expected = expected_from_func_call(FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![Expression::Boolean(true)],
    });
    assert_eq!(ast.0, expected);

    let ast: Ast = "(foo 1 (bar 2 false))".parse().unwrap();
    let expected = expected_from_func_call(FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![
            Expression::Number(1.0),
            Expression::Function(FunctionExpression::Function(FunctionCall {
                identifier: Identifier("bar".to_string()),
                parameters: vec![
                    Expression::Number(2.0),
                    Expression::Boolean(false),
                ],
            })),
        ],
    });
    assert_eq!(ast.0, expected);

    fn expected_from_func_call(value: FunctionCall) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            Expression::Function(FunctionExpression::Function(value)),
        ))]
    }
}

#[test]
fn test_parse_function_list() {
    let ast: Ast = "[foo, bar]".parse().unwrap();
    let expected = expected_from_func_calls(vec![
        FunctionCall {
            identifier: Identifier("foo".to_string()),
            parameters: Vec::new(),
        },
        FunctionCall {
            identifier: Identifier("bar".to_string()),
            parameters: Vec::new(),
        },
    ]);
    assert_eq!(ast.0, expected);

    let ast: Ast =
        "[foo, (bar true), (baz 1 (waldo 2 3 [fred, (corge false)]))]"
            .parse()
            .unwrap();
    let expected = expected_from_func_calls(vec![
        FunctionCall {
            identifier: Identifier("foo".to_string()),
            parameters: Vec::new(),
        },
        FunctionCall {
            identifier: Identifier("bar".to_string()),
            parameters: vec![Expression::Boolean(true)],
        },
        FunctionCall {
            identifier: Identifier("baz".to_string()),
            parameters: vec![
                Expression::Number(1.0),
                expression_from_func_call(FunctionCall {
                    identifier: Identifier("waldo".to_string()),
                    parameters: vec![
                        Expression::Number(2.0),
                        Expression::Number(3.0),
                        expression_from_func_calls(vec![
                            //ohmy
                            FunctionCall {
                                identifier: Identifier("fred".to_string()),
                                parameters: Vec::new(),
                            },
                            FunctionCall {
                                identifier: Identifier("corge".to_string()),
                                parameters: vec![Expression::Boolean(false)],
                            },
                        ]),
                    ],
                }),
            ],
        },
    ]);
    assert_eq!(ast.0, expected);

    fn expected_from_func_calls(value: Vec<FunctionCall>) -> Vec<Statement> {
        vec![Statement::SuperExpression(SuperExpression::Expression(
            Expression::Function(FunctionExpression::FunctionList(value)),
        ))]
    }
}

#[test]
fn test_parse_array() {
    let ast: Ast = "[foo, 1, true, \"hello\", [1, 2], $16.19, 1.234]"
        .parse()
        .unwrap();
    let expected = vec![Statement::SuperExpression(
        SuperExpression::Expression(Expression::Array(vec![
            superexpression_from_expression(expression_from_func_call(
                FunctionCall {
                    identifier: Identifier("foo".to_string()),
                    parameters: Vec::new(),
                },
            )),
            superexpression_from_expression(Expression::Number(1.0)),
            superexpression_from_expression(Expression::Boolean(true)),
            superexpression_from_expression(Expression::String(
                "hello".to_string(),
            )),
            superexpression_from_expression(Expression::Array(vec![
                superexpression_from_expression(Expression::Number(1.0)),
                superexpression_from_expression(Expression::Number(2.0)),
            ])),
            superexpression_from_expression(Expression::PatternSlot((16, 19))),
            superexpression_from_expression(Expression::Number(1.234)),
        ])),
    )];
    assert_eq!(ast.0, expected);
}

#[allow(dead_code)]
fn expression_from_func_call(func_call: FunctionCall) -> Expression {
    Expression::Function(FunctionExpression::Function(func_call))
}

#[allow(dead_code)]
fn expression_from_func_calls(func_calls: Vec<FunctionCall>) -> Expression {
    Expression::Function(FunctionExpression::FunctionList(func_calls))
}

#[allow(dead_code)]
fn superexpression_from_expression(expr: Expression) -> SuperExpression {
    SuperExpression::Expression(expr)
}
