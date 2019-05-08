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
        property_id: vec![Identifier("bar".into())],
    };
    let result: ParseResult<Expression> =
        parse_source_for_rule(":foo.bar", Rule::Expression);
    assert_eq!(expected, result.unwrap());

    let expected = Expression::PropertyGetter {
        assignee: Box::new(Expression::Variable(Identifier("foo".into()))),
        property_id: vec![
            Identifier("bar".into()),
            Identifier("baz".into()),
            Identifier("fred".into()),
        ],
    };
    let result: ParseResult<Expression> =
        parse_source_for_rule(":foo.bar.baz.fred", Rule::Expression);
    assert_eq!(expected, result.unwrap());
}

#[test]
fn test_parse_track() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("$0", Rule::Expression);
    assert_eq!(Expression::Track(0), result.unwrap());
}

#[test]
fn test_parse_pattern_slot() {
    let result: ParseResult<Expression> =
        parse_source_for_rule("$0.1", Rule::Expression);
    assert_eq!(Expression::PatternSlot((0, 1)), result.unwrap());
}

#[test]
fn test_parse_function_expression() {
    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: Vec::new(),
    };
    let result: ParseResult<FunctionCall> =
        parse_source_for_rule("foo", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());

    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![FunctionCall {
            identifier: Identifier("bar".to_string()),
            parameters: Vec::new(),
        }
        .into()],
    };
    let result: ParseResult<FunctionCall> =
        parse_source_for_rule("(foo bar)", Rule::FunctionCall);
    assert_eq!(expected, result.unwrap());

    let ast: Ast = "(foo true)".parse().unwrap();
    let expected = FunctionCall {
        identifier: Identifier("foo".to_string()),
        parameters: vec![Expression::Boolean(true)],
    };
    let result: ParseResult<FunctionCall> =
        parse_source_for_rule("(foo true)", Rule::FunctionCall);
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
    let result: ParseResult<FunctionCall> =
        parse_source_for_rule("(foo 1 (bar 2 false))", Rule::FunctionCall);
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
                FunctionCall {
                    identifier: Identifier("waldo".to_string()),
                    parameters: vec![
                        Expression::Number(2.0),
                        Expression::Number(3.0),
                        vec![
                            //ohmy
                            FunctionCall {
                                identifier: Identifier("fred".to_string()),
                                parameters: Vec::new(),
                            },
                            FunctionCall {
                                identifier: Identifier("corge".to_string()),
                                parameters: vec![Expression::Boolean(false)],
                            },
                        ]
                        .into(),
                    ],
                }
                .into(),
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
        Expression::Array(vec![
            SuperExpression::Expression(
                FunctionCall {
                    identifier: Identifier("foo".to_string()),
                    parameters: Vec::new(),
                }
                .into(),
            ),
            Expression::Number(1.0).into(),
            Expression::Boolean(true).into(),
            Expression::String("hello".to_string()).into(),
            Expression::Array(vec![
                Expression::Number(1.0).into(),
                Expression::Number(2.0).into(),
            ])
            .into(),
            Expression::PatternSlot((16, 19)).into(),
            Expression::Number(1.234).into(),
        ])
        .into(),
    )];
    assert_eq!(ast.0, expected);
}

#[test]
fn test_parse_properties() {
    let source = "{foo: true, bar: \"hello\", baz: 1.0, fred: foo, ringo: [1, false], paul: {foo: 1, bar: false}}";
    let result: ParseResult<Properties> =
        parse_source_for_rule(source, Rule::Properties);
    let mut map: HashMap<Identifier, PropertyValue> = HashMap::new();
    map.insert(
        Identifier("foo".into()),
        PropertyValue::SuperExpression(Expression::Boolean(true).into()),
    );
    map.insert(
        Identifier("bar".into()),
        PropertyValue::SuperExpression(
            Expression::String("hello".into()).into(),
        ),
    );
    map.insert(
        Identifier("baz".into()),
        PropertyValue::SuperExpression(Expression::Number(1.0).into()),
    );
    map.insert(
        Identifier("fred".into()),
        PropertyValue::SuperExpression(SuperExpression::Expression(
            FunctionCall {
                identifier: Identifier("foo".into()),
                parameters: Vec::new(),
            }
            .into(),
        )),
    );
    map.insert(
        Identifier("ringo".into()),
        PropertyValue::SuperExpression(
            Expression::Array(vec![
                Expression::Number(1.0).into(),
                Expression::Boolean(false).into(),
            ])
            .into(),
        ),
    );

    let mut inner_properties: HashMap<Identifier, PropertyValue> =
        HashMap::new();
    inner_properties.insert(
        Identifier("foo".into()),
        PropertyValue::SuperExpression(Expression::Number(1.0).into()),
    );
    inner_properties.insert(
        Identifier("bar".into()),
        PropertyValue::SuperExpression(Expression::Boolean(false).into()),
    );
    map.insert(
        Identifier("paul".into()),
        PropertyValue::SuperExpression(
            Expression::Properties(Properties(inner_properties)).into(),
        ),
    );

    assert_eq!(Properties(map), result.unwrap());
}

#[allow(dead_code)]
impl From<FunctionCall> for Expression {
    fn from(func_call: FunctionCall) -> Self {
        Expression::Function(FunctionExpression::Function(func_call))
    }
}

#[allow(dead_code)]
impl From<Vec<FunctionCall>> for Expression {
    fn from(func_calls: Vec<FunctionCall>) -> Self {
        Expression::Function(FunctionExpression::FunctionList(func_calls))
    }
}

#[allow(dead_code)]
impl From<Expression> for SuperExpression {
    fn from(expression: Expression) -> Self {
        SuperExpression::Expression(expression)
    }
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
