use super::*;

#[test]
fn test_parse_number() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("0.123", Rule::Expression);
    assert_eq!(Expression::Number(0.123), result.unwrap());

    let result: ParseResult<Expression> =
        parse_source_for_rule("10", Rule::Expression);
    assert_eq!(Expression::Number(10.0), result.unwrap());

    let result: ParseResult<Expression> =
        parse_source_for_rule("1e5", Rule::Expression);
    assert_eq!(Expression::Number(1e5), result.unwrap());

    let result: ParseResult<Expression> =
        parse_source_for_rule("1e-5", Rule::Expression);
    assert_eq!(Expression::Number(1e-5), result.unwrap());
}

#[test]
fn test_parse_boolean() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("true", Rule::Expression);
    assert_eq!(Expression::Boolean(true), result.unwrap());

    let result: ParseResult<Expression> =
        parse_source_for_rule("false", Rule::Expression);
    assert_eq!(Expression::Boolean(false), result.unwrap());
}

#[test]
fn test_parse_string() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("\"hello world {} \n\"", Rule::Expression);
    assert_eq!(
        Expression::String("hello world {} \n".to_string()),
        result.unwrap()
    );
}

#[test]
fn test_parse_mixer() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("$", Rule::Expression);
    assert_eq!(Expression::Mixer, result.unwrap());
}

#[test]
fn test_parse_variable() {
    let result: ParseResult<Expression> =
        parse_source_for_rule(":foo", Rule::Expression);
    assert_eq!(
        Expression::Variable(Identifier("foo".to_string())),
        result.unwrap()
    );
}

#[test]
fn test_parse_property_getter() {
    let expected = Expression::PropertyGetter {
        assignee: Box::new(Expression::Variable(Identifier("foo".into()))),
        property_id: Identifier("bar".into()),
    };
    let result: ParseResult<Expression> = parse_source_for_rule(":foo.bar", Rule::Expression);
    assert_eq!(expected, result.unwrap());

    //TODO
    // let ast: Ast = ":foo.bar.baz.fred".parse().unwrap();
}

#[test]
fn test_parse_track() {
    let result: ParseResult<Expression> = parse_source_for_rule("$0", Rule::Expression);
    assert_eq!(Expression::Track(0), result.unwrap());
}

#[test]
fn test_parse_pattern_slot() {
    let result: ParseResult<Expression> = parse_source_for_rule("$0.1", Rule::Expression);
    assert_eq!(Expression::PatternSlot((0, 1)), result.unwrap());
}

#[test]
fn test_parse_function_expression() {
    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: Vec::new(),
    };
    let result: ParseResult<FunctionCall> = parse_source_for_rule("foo", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());

    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![expression_from_func_call(FunctionCall {
            identifier: Identifier("bar".to_string()),
            parameters: Vec::new(),
        })],
    };
    let result: ParseResult<FunctionCall> = parse_source_for_rule("(foo bar)", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());

    let ast: Ast = "(foo true)".parse().unwrap();
    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![Expression::Boolean(true)],
    };
    let result: ParseResult<FunctionCall> = parse_source_for_rule("(foo true)", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());

    let ast: Ast = "(foo 1 (bar 2 false))".parse().unwrap();
    let expected = FunctionCall {
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
    };
    let result: ParseResult<FunctionCall> = parse_source_for_rule("(foo 1 (bar 2 false))", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());
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

#[allow(dead_code)]
fn parse_source_for_rule<'a, T>(
    source: &'a str,
    rule: Rule,
) -> Result<T, T::Error>
where
    T: TryFrom<Pair<'a, Rule>>,
{
    let pair = CollyParser::parse(rule, source).unwrap().peek().unwrap();
    T::try_from(pair)
}
