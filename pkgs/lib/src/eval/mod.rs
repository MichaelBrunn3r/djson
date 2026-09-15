#![allow(
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation
)]

pub mod document;
pub mod scope;
pub mod stdlib;

pub use document::{Document, Value};
use miette::SourceSpan;
pub use scope::Scope;

use crate::{
    eval::document::map::Map,
    parser::ast::{AST, Expr, Identifier, InfixOp, Pattern, PrefixOp, Statement},
    span::Spanned,
};

/// Evaluates a parsed document into `scope`.
///
/// Bindings declared by `let` statements are written directly into `scope`.
/// Callers that need an isolated evaluation environment, such as normal file
/// evaluation, should pass a child scope created from the standard prelude.
pub fn evaluate_ast(ast: &AST<'_>, scope: &mut Scope) -> Result<Value, EvalError> {
    let mut document = Document::new();
    let mut expression = None;

    for statement in &ast.statements {
        let value = match &statement.value {
            Statement::Let(Spanned { value: binding, .. }) => {
                let value = evaluate_expr(&binding.expr, scope)?;
                let mut bindings = Vec::new();
                destructure(&binding.pattern, value, scope, &mut bindings)?;
                commit_bindings(scope, &bindings)?;
                continue;
            }
            Statement::Expr(node) => {
                if !document.is_empty() {
                    return Err(EvalError::MixedDocumentForms {
                        span: statement.span,
                    });
                }
                if expression.is_some() {
                    return Err(EvalError::MultipleExpressions {
                        span: statement.span,
                    });
                }
                expression = Some(evaluate_expr(node, scope)?);
                continue;
            }
            Statement::KV(Spanned { value: pair, .. }) => {
                if expression.is_some() {
                    return Err(EvalError::MixedDocumentForms {
                        span: statement.span,
                    });
                }
                evaluate_expr(&pair.expr, scope)?
            }
        };

        if let Statement::KV(Spanned { value: pair, .. }) = &statement.value
            && document.insert(pair.key.value, value).is_some()
        {
            return Err(EvalError::DuplicateKey {
                key: pair.key.value.to_owned(),
                span: pair.key.span,
            });
        }
    }

    Ok(expression.unwrap_or(Value::Map(document)))
}

/// A destructured binding, paired with the span of the name that declares it.
struct Binding<'input> {
    name: &'input str,
    span: SourceSpan,
    value: Value,
}

fn destructure<'input>(
    pattern: &Spanned<Pattern<'input>>,
    value: Value,
    scope: &Scope,
    bindings: &mut Vec<Binding<'input>>,
) -> Result<(), EvalError> {
    match &pattern.value {
        Pattern::Name(name) => bindings.push(Binding {
            name,
            span: pattern.span,
            value,
        }),
        Pattern::Map(patterns) => {
            let Value::Map(map) = value else {
                return Err(EvalError::PatternMismatch { span: pattern.span });
            };
            for map_pattern in patterns {
                let pattern = &map_pattern.value;
                let value = match map.get(pattern.key).cloned() {
                    Some(value) => value,
                    None => match (&pattern.pattern.value, &pattern.default) {
                        (Pattern::Name(_), Some(default)) => evaluate_expr(default, scope)?,
                        (Pattern::Map(_), None) => Value::Map(Map::new()),
                        _ => {
                            return Err(EvalError::PatternMismatch {
                                span: map_pattern.span,
                            });
                        }
                    },
                };
                destructure(&pattern.pattern, value, scope, bindings)?;
            }
        }
        Pattern::List { patterns, rest } => {
            let Value::List(values) = value else {
                return Err(EvalError::PatternMismatch { span: pattern.span });
            };
            if patterns.len() > values.len() {
                return Err(EvalError::PatternMismatch { span: pattern.span });
            }
            for (pattern, value) in patterns.iter().zip(values.iter().cloned()) {
                destructure(pattern, value, scope, bindings)?;
            }
            if let Some(rest) = rest {
                bindings.push(Binding {
                    name: rest.value,
                    span: rest.span,
                    value: Value::List(values.into_iter().skip(patterns.len()).collect()),
                });
            }
        }
    }
    Ok(())
}

/// Binds destructured names into `scope`.
///
/// [`Scope::bind_values`] reports conflicts by name only, so the binding that
/// declares the conflicting name is looked up again to label its span.
fn commit_bindings(scope: &mut Scope, bindings: &[Binding<'_>]) -> Result<(), EvalError> {
    scope
        .bind_values(
            bindings
                .iter()
                .map(|binding| (binding.name.to_owned(), binding.value.clone())),
        )
        .map_err(|error| match error {
            EvalError::SymbolConflict { name, .. } => {
                let binding = bindings.iter().rev().find(|binding| binding.name == name);
                EvalError::SymbolConflict {
                    name,
                    span: binding.map(|binding| binding.span),
                }
            }
            error => error,
        })
}

fn evaluate_expr(node: &Spanned<Expr<'_>>, scope: &Scope) -> Result<Value, EvalError> {
    let span = node.span;
    match &node.value {
        Expr::Bool(value) => Ok(Value::Bool(*value)),
        Expr::Int(value) => Ok(Value::Int(*value)),
        Expr::Float(value) => Ok(Value::Float(*value)),
        Expr::Str(value) => Ok(Value::Str(decode_string(value))),
        Expr::Id(Identifier::Simple("none" | "null" | "nil")) => Ok(Value::None),
        Expr::List(values) => values
            .iter()
            .map(|value| evaluate_expr(value, scope))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List),
        Expr::Map(entries) => {
            let mut map = Map::new();
            for entry in entries {
                let entry = &entry.value;
                let value = evaluate_expr(&entry.expr, scope)?;
                if map.insert(entry.key.value, value).is_some() {
                    return Err(EvalError::DuplicateKey {
                        key: entry.key.value.to_owned(),
                        span: entry.key.span,
                    });
                }
            }
            Ok(Value::Map(map))
        }
        Expr::Access { object, name } => evaluate_expr(object, scope)
            .and_then(|object| resolve_member(&object, name, span))
            .map(|(value, _)| value)
            .map_err(|error| error.at(span)),
        Expr::Id(identifier) => {
            let Identifier::Simple(name) = identifier;
            scope
                .resolve(name)
                .ok_or_else(|| EvalError::UnknownIdentifier {
                    name: (*name).to_owned(),
                    span,
                })
        }
        Expr::Unary { op, value } => {
            evaluate_unary(op, evaluate_expr(value, scope)?).map_err(|error| error.at(span))
        }
        Expr::Binary { left, op, right } => evaluate_binary(
            op,
            evaluate_expr(left, scope)?,
            evaluate_expr(right, scope)?,
        )
        .map_err(|error| error.at(span)),
        Expr::Call { callee, arguments } => {
            evaluate_call(callee, arguments, scope).map_err(|error| error.at(span))
        }
        Expr::If {
            condition,
            then: then_branch,
            r#else: else_branch,
        } => {
            let Value::Bool(test) = evaluate_expr(condition, scope)? else {
                return Err(EvalError::TypeMismatch { span: None }.at(condition.span));
            };
            if test {
                evaluate_expr(then_branch, scope)
            } else {
                evaluate_expr(else_branch, scope)
            }
        }
    }
}

fn decode_string(value: &str) -> String {
    let mut decoded = String::with_capacity(value.len());
    let mut characters = value.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            decoded.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some('t') => decoded.push('\t'),
            Some('\\') | None => decoded.push('\\'),
            Some('"') => decoded.push('"'),
            Some('\'') => decoded.push('\''),
            Some(next) => {
                decoded.push('\\');
                decoded.push(next);
            }
        }
    }

    decoded
}

fn resolve_member(
    object: &Value,
    name: &str,
    span: SourceSpan,
) -> Result<(Value, bool), EvalError> {
    match object {
        Value::Map(map) => map
            .get(name)
            .cloned()
            .map(|value| (value, false))
            .or_else(|| stdlib::type_member(object, name).map(|value| (value, true)))
            .ok_or_else(|| EvalError::UnknownIdentifier {
                name: name.to_owned(),
                span,
            }),
        value => stdlib::type_member(value, name)
            .map(|value| (value, true))
            .ok_or_else(|| EvalError::UnknownIdentifier {
                name: name.to_owned(),
                span,
            }),
    }
}

/// Rewrites a failed member lookup into a failed call.
fn to_unknown_function(error: EvalError) -> EvalError {
    match error {
        EvalError::UnknownIdentifier { name, span } => EvalError::UnknownFunction { name, span },
        error => error,
    }
}

fn evaluate_call(
    callee: &Spanned<Expr<'_>>,
    arguments: &[Spanned<Expr<'_>>],
    scope: &Scope,
) -> Result<Value, EvalError> {
    let mut receiver = None;
    let value = match &callee.value {
        Expr::Access { object, name } => {
            let object = evaluate_expr(object, scope).map_err(to_unknown_function)?;
            let (value, binds_receiver) =
                resolve_member(&object, name, callee.span).map_err(to_unknown_function)?;
            if binds_receiver {
                receiver = Some(object);
            }
            value
        }
        _ => evaluate_expr(callee, scope).map_err(to_unknown_function)?,
    };
    let Value::Function(function) = value else {
        return Err(EvalError::NotCallable { span: callee.span });
    };
    let mut arguments = arguments
        .iter()
        .map(|argument| evaluate_expr(argument, scope))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(receiver) = receiver {
        arguments.insert(0, receiver);
    }

    function(&arguments)
}

fn evaluate_unary(operator: &PrefixOp, value: Value) -> Result<Value, EvalError> {
    match (operator, value) {
        (PrefixOp::Positive, value @ (Value::Int(_) | Value::Float(_))) => Ok(value),
        (PrefixOp::Negative, Value::Int(value)) => value
            .checked_neg()
            .map(Value::Int)
            .ok_or(EvalError::Overflow { span: None }),
        (PrefixOp::Negative, Value::Float(value)) => Ok(Value::Float(-value)),
        (PrefixOp::Positive | PrefixOp::Negative, _) => Err(EvalError::TypeMismatch { span: None }),
    }
}

fn evaluate_binary(op: &InfixOp, left: Value, right: Value) -> Result<Value, EvalError> {
    match (op, left, right) {
        (InfixOp::Add, Value::Int(left), Value::Int(right)) => left
            .checked_add(right)
            .map(Value::Int)
            .ok_or(EvalError::Overflow { span: None }),
        (InfixOp::Sub, Value::Int(left), Value::Int(right)) => left
            .checked_sub(right)
            .map(Value::Int)
            .ok_or(EvalError::Overflow { span: None }),
        (InfixOp::Mul, Value::Int(left), Value::Int(right)) => left
            .checked_mul(right)
            .map(Value::Int)
            .ok_or(EvalError::Overflow { span: None }),
        (InfixOp::Div, Value::Int(_), Value::Int(0))
        | (InfixOp::Div, Value::Float(_), Value::Float(0.0)) => {
            Err(EvalError::DivisionByZero { span: None })
        }
        (InfixOp::Div, Value::Int(left), Value::Int(right)) => left
            .checked_div(right)
            .map(Value::Int)
            .ok_or(EvalError::Overflow { span: None }),
        (InfixOp::Exp, Value::Int(left), Value::Int(right)) if right >= 0 => {
            let exponent = u32::try_from(right).map_err(|_| EvalError::Overflow { span: None })?;
            left.checked_pow(exponent)
                .map(Value::Int)
                .ok_or(EvalError::Overflow { span: None })
        }
        (InfixOp::Add, Value::Float(left), Value::Float(right)) => Ok(Value::Float(left + right)),
        (InfixOp::Sub, Value::Float(left), Value::Float(right)) => Ok(Value::Float(left - right)),
        (InfixOp::Mul, Value::Float(left), Value::Float(right)) => Ok(Value::Float(left * right)),
        (InfixOp::Div, Value::Float(left), Value::Float(right)) => Ok(Value::Float(left / right)),
        (InfixOp::Exp, Value::Float(left), Value::Float(right)) => {
            Ok(Value::Float(left.powf(right)))
        }
        (InfixOp::Equal, left, right)
            if matches!(
                (&left, &right),
                (
                    Value::None | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Str(_),
                    Value::None | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Str(_),
                )
            ) =>
        {
            Ok(Value::Bool(left == right))
        }
        (operator, Value::Int(left), Value::Float(right)) => {
            evaluate_binary(operator, Value::Float(left as f64), Value::Float(right))
        }
        (operator, Value::Float(left), Value::Int(right)) => {
            evaluate_binary(operator, Value::Float(left), Value::Float(right as f64))
        }
        _ => Err(EvalError::TypeMismatch { span: None }),
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, miette::Diagnostic)]
pub enum EvalError {
    #[error("assertion failed")]
    #[diagnostic(code(eval::assertion_failed))]
    AssertionFailed {
        #[label("assertion is false")]
        span: Option<SourceSpan>,
    },

    #[error("duplicate key `{key}`")]
    #[diagnostic(code(eval::duplicate_key))]
    DuplicateKey {
        key: String,
        #[label("`{key}` is already defined by an earlier entry")]
        span: SourceSpan,
    },

    #[error("division by zero")]
    #[diagnostic(code(eval::division_by_zero))]
    DivisionByZero {
        #[label("divisor is zero")]
        span: Option<SourceSpan>,
    },

    #[error("mixed document forms")]
    #[diagnostic(code(eval::mixed_document_forms))]
    MixedDocumentForms {
        #[label("a document holds either entries or a single expression")]
        span: SourceSpan,
    },

    #[error("multiple expressions")]
    #[diagnostic(code(eval::multiple_expressions))]
    MultipleExpressions {
        #[label("a document holds at most one expression")]
        span: SourceSpan,
    },

    #[error("numeric overflow")]
    #[diagnostic(code(eval::overflow))]
    Overflow {
        #[label("this operation overflows")]
        span: Option<SourceSpan>,
    },

    #[error("pattern mismatch")]
    #[diagnostic(code(eval::pattern_mismatch))]
    PatternMismatch {
        #[label("pattern does not match the value")]
        span: SourceSpan,
    },

    #[error("type mismatch")]
    #[diagnostic(code(eval::type_mismatch))]
    TypeMismatch {
        #[label("unexpected type")]
        span: Option<SourceSpan>,
    },

    #[error("unknown identifier `{name}`")]
    #[diagnostic(code(eval::unknown_identifier))]
    UnknownIdentifier {
        name: String,
        #[label("`{name}` is not bound")]
        span: SourceSpan,
    },

    #[error("unknown function `{name}`")]
    #[diagnostic(code(eval::unknown_function))]
    UnknownFunction {
        name: String,
        #[label("`{name}` is not a function")]
        span: SourceSpan,
    },

    #[error("unknown module `{name}`")]
    #[diagnostic(code(eval::unknown_module))]
    UnknownModule {
        name: String,
        #[label("no builtin module is named `{name}`")]
        span: Option<SourceSpan>,
    },

    #[error("value is not callable")]
    #[diagnostic(code(eval::not_callable))]
    NotCallable {
        #[label("this value is not a function")]
        span: SourceSpan,
    },

    #[error("`{name}` is already bound")]
    #[diagnostic(code(eval::symbol_conflict))]
    SymbolConflict {
        name: String,
        #[label("`{name}` is already bound in this scope")]
        span: Option<SourceSpan>,
    },
}

impl EvalError {
    /// Attaches `span` to an error that carries no source position yet.
    ///
    /// Builtins, [`Scope`], and operator evaluation report errors without a
    /// source position, so the evaluator calls this with the span of the
    /// expression it was evaluating. Spans attached closer to the offending
    /// expression take precedence, and variants the evaluator always builds
    /// with a span are returned unchanged.
    #[must_use]
    pub fn at(self, span: SourceSpan) -> Self {
        let span = Some(span);
        match self {
            Self::AssertionFailed { span: current } => Self::AssertionFailed {
                span: current.or(span),
            },
            Self::DivisionByZero { span: current } => Self::DivisionByZero {
                span: current.or(span),
            },
            Self::Overflow { span: current } => Self::Overflow {
                span: current.or(span),
            },
            Self::TypeMismatch { span: current } => Self::TypeMismatch {
                span: current.or(span),
            },
            Self::UnknownModule {
                name,
                span: current,
            } => Self::UnknownModule {
                name,
                span: current.or(span),
            },
            Self::SymbolConflict {
                name,
                span: current,
            } => Self::SymbolConflict {
                name,
                span: current.or(span),
            },
            error @ (Self::DuplicateKey { .. }
            | Self::MixedDocumentForms { .. }
            | Self::MultipleExpressions { .. }
            | Self::NotCallable { .. }
            | Self::PatternMismatch { .. }
            | Self::UnknownFunction { .. }
            | Self::UnknownIdentifier { .. }) => error,
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use miette::Diagnostic;

    use super::test_utils::*;
    use super::*;
    use crate::{
        map,
        parser::Parser,
        test_utils::{fmt_diagnostic_case, fmt_snapshot_case, fmt_snapshot_cases},
        value,
    };

    fn evaluate(input: &str) -> Result<Value, EvalError> {
        let ast = Parser::new(input).parse_stmnts().expect("valid input");
        let mut scope = Scope::child(stdlib::new());
        evaluate_ast(&ast, &mut scope)
    }

    /// Evaluates `input` and renders the error it reports.
    fn evaluate_error(input: &str) -> String {
        evaluate(input)
            .expect_err("expected an evaluation error")
            .to_string()
    }

    /// Offsets and lengths of the labels attached to `error`.
    fn label_spans(error: &EvalError) -> Vec<(usize, usize)> {
        let Some(labels) = error.labels() else {
            panic!("evaluation errors carry labels");
        };
        labels.map(|label| (label.offset(), label.len())).collect()
    }

    fn expect_document(label: &str, input: &str, expected: &[(&str, Value)]) {
        let value = evaluate(input)
            .unwrap_or_else(|error| panic!("{label}: expected valid document, got {error}"));
        let Value::Map(document) = value else {
            panic!("{label}: expected map value")
        };

        assert_entries(&document, expected);
    }

    #[test]
    fn evaluates_documents() {
        let cases = vec![
            ("literal entries", "count: 3", vec![("count", value!(3))]),
            (
                "boolean literal",
                "enabled: true",
                vec![("enabled", value!(true))],
            ),
            (
                "none aliases",
                "none_value: none\nnull_value: null\nnil_value: nil",
                vec![
                    ("none_value", value!(none)),
                    ("null_value", value!(null)),
                    ("nil_value", value!(nil)),
                ],
            ),
            (
                "nested numeric expression",
                "result: 1 + 2 * 3",
                vec![("result", value!(7))],
            ),
            (
                "unary and mixed numeric expressions",
                "negative: -2\nmixed: 1 + 2.5",
                vec![("negative", value!(-2)), ("mixed", value!(3.5))],
            ),
        ];

        for (label, input, expected) in cases {
            expect_document(label, input, &expected);
        }
    }

    #[test]
    fn evaluates_map_literals_and_member_access() {
        assert_eq!(
            evaluate("{outer: {value: 7}, \"quoted-key\": true}.outer.value"),
            Ok(Value::Int(7))
        );
        assert_eq!(
            evaluate("{\"quoted-key\": true}"),
            Ok(Value::Map(map! { "quoted-key": true }))
        );
        assert_eq!(evaluate("{a: 1,}.a"), Ok(value!(1)));
    }

    #[test]
    fn evaluates_destructuring_patterns() {
        assert_eq!(
            evaluate(
                "let {outer.{a, inner.{b}}} = {outer: {a: 1, inner: {b: 2}}}
                 result: a + b"
            ),
            Ok(Value::Map(map! { "result": 3 }))
        );
        assert_eq!(
            evaluate("let {assert} = import(\"std.assert\")\nassert(true)"),
            Ok(Value::Bool(true))
        );
        assert_eq!(
            evaluate("let [a, b] = [1, 2]\nresult: a + b"),
            Ok(Value::Map(map! { "result": 3 }))
        );
        assert_eq!(
            evaluate("let [a] = [1, 2, 3]\nresult: a"),
            Ok(Value::Map(map! { "result": 1 }))
        );
        assert_eq!(
            evaluate("let [a, ..rest] = [1, 2, 3]\nresult: rest"),
            Ok(Value::Map(Map::from([(
                "result",
                Value::List(vec![value!(2), value!(3)]),
            )])))
        );
        assert_eq!(
            evaluate("let [..values] = [1, 2]\nresult: values"),
            Ok(Value::Map(Map::from([(
                "result",
                Value::List(vec![value!(1), value!(2)]),
            )])))
        );
        assert_eq!(
            evaluate("let [a, ..rest] = [1]\nresult: rest"),
            Ok(Value::Map(Map::from([
                ("result", Value::List(Vec::new()),)
            ])))
        );
        assert_eq!(
            evaluate("let {x = 1 + 2} = {y: 2}\nresult: x"),
            Ok(Value::Map(map! { "result": 3 }))
        );
        assert_eq!(
            evaluate("let {x = 10} = {x: none}\nresult: x"),
            Ok(Value::Map(Map::from([("result", Value::None)])))
        );
        assert_eq!(
            evaluate("let {x = missing} = {x: 1}\nresult: x"),
            Ok(Value::Map(map! { "result": 1 }))
        );
        assert_eq!(
            evaluate("let {a.{b.{c.{d.{e = 10}}}}} = {a: {b: {}}}\nresult: e"),
            Ok(Value::Map(map! { "result": 10 }))
        );
    }

    #[test]
    fn rejects_list_patterns_longer_than_values() {
        assert_eq!(evaluate_error("let [a, b] = [1]"), "pattern mismatch");
    }

    #[test]
    fn rejects_unmatched_patterns() {
        assert_eq!(evaluate_error("let {x} = {y: 1}"), "pattern mismatch");
        assert_eq!(
            evaluate_error("let {nested.{x}} = {nested: 1}"),
            "pattern mismatch"
        );
        assert_eq!(evaluate_error("let [x] = {x: 1}"), "pattern mismatch");
    }

    #[test]
    fn binds_let_from_a_map_pattern() {
        let ast = Parser::new("let {let} = {let: 7}")
            .parse_stmnts()
            .expect("valid destructuring pattern");
        let mut scope = Scope::child(stdlib::new());

        assert_eq!(evaluate_ast(&ast, &mut scope), Ok(Value::Map(map! {})));
        assert_eq!(scope.resolve("let"), Some(Value::Int(7)));

        let ast = Parser::new("let {let.{let}} = {let: {let: 9}}")
            .parse_stmnts()
            .expect("valid nested pattern");
        let mut scope = Scope::child(stdlib::new());

        assert_eq!(evaluate_ast(&ast, &mut scope), Ok(Value::Map(map! {})));
        assert_eq!(scope.resolve("let"), Some(Value::Int(9)));

        let ast = Parser::new("let let = 8")
            .parse_stmnts()
            .expect("valid name pattern");
        let mut scope = Scope::child(stdlib::new());

        assert_eq!(evaluate_ast(&ast, &mut scope), Ok(Value::Map(map! {})));
        assert_eq!(scope.resolve("let"), Some(Value::Int(8)));
    }

    #[test]
    fn rejects_duplicate_destructured_names_without_partial_bindings() {
        let ast = Parser::new("let {a, nested.{a}} = {a: 1, nested: {a: 2}}")
            .parse_stmnts()
            .expect("valid destructuring pattern");
        let mut scope = Scope::child(stdlib::new());

        let error = evaluate_ast(&ast, &mut scope).expect_err("expected a symbol conflict");

        assert_eq!(error.to_string(), "`a` is already bound");
        assert_eq!(label_spans(&error), vec![(16, 1)]);
        assert_eq!(scope.resolve("a"), None);
    }

    #[test]
    fn evaluates_none_aliases_as_equal() {
        assert_eq!(evaluate("none == null"), Ok(value!(true)));
        assert_eq!(evaluate("null == nil"), Ok(value!(true)));
    }

    #[test]
    fn rejects_duplicate_map_keys() {
        assert_eq!(
            evaluate_error("{value: 1, value: 2}"),
            "duplicate key `value`"
        );
    }

    #[test]
    fn evaluates_expression_documents_to_values() {
        assert_eq!(evaluate("1 + 2 * 3"), Ok(value!(7)));
        assert_eq!(evaluate("count: 3"), Ok(Value::Map(map! { count: 3 })));
        assert_eq!(evaluate("[1, 2 * 3, [4, 5]]"), Ok(value!([1, 6, [4, 5]])));
    }

    #[test]
    fn evaluates_if_else_expressions_lazily() {
        assert_eq!(evaluate("if true { 1 } else { missing }"), Ok(value!(1)));
        assert_eq!(evaluate("if false { missing } else { 2 }"), Ok(value!(2)));
        assert_eq!(evaluate("if 1 == 1 { 1 } else { 2 }"), Ok(value!(1)));
        assert_eq!(evaluate("if (true) { 1 } else { missing }"), Ok(value!(1)));
        assert_eq!(evaluate("if (false) { missing } else { 2 }"), Ok(value!(2)));
        assert_eq!(evaluate_error("if (1) { 1 } else { 2 }"), "type mismatch");
    }

    #[test]
    fn evaluates_let_bindings_without_document_fields() {
        assert_eq!(
            evaluate("let value = { nested: 7 }\nresult: value.nested"),
            Ok(Value::Map(map! { result: 7 }))
        );
    }

    #[test]
    fn imports_standard_library_as_a_map() {
        assert_eq!(
            evaluate("let std = import(\"std\")\nstd.math.sin(std.math.PI / 2)"),
            Ok(value!(1.0))
        );
        assert_eq!(
            evaluate("let std = import(\"std\")\nstd.types.int.sqrt(9)"),
            Ok(value!(3.0))
        );
    }

    #[test]
    fn rejects_unknown_and_invalid_imports() {
        assert_eq!(
            evaluate_error("import(\"missing\")"),
            "unknown module `missing`"
        );
        assert_eq!(evaluate_error("import(1)"), "type mismatch");
    }

    #[test]
    fn imports_nested_modules_by_dotted_path() {
        assert_eq!(
            evaluate("let assert = import(\"std.assert\")\nassert.assert(true)"),
            Ok(value!(true))
        );
        assert_eq!(
            evaluate("let int = import(\"types.int\")\nint.sqrt(9)"),
            Ok(value!(3.0))
        );
        assert_eq!(
            evaluate_error("import(\"std.missing\")"),
            "unknown module `std.missing`"
        );
    }

    #[test]
    fn rejects_duplicate_let_bindings() {
        assert_eq!(
            evaluate_error("let value = 1\nlet value = 2"),
            "`value` is already bound"
        );
    }

    #[test]
    fn evaluates_strict_scalar_equality() {
        assert_eq!(evaluate("1 == 1"), Ok(value!(true)));
        assert_eq!(evaluate("1 == 2"), Ok(value!(false)));
        assert_eq!(evaluate("1 == 1.0"), Ok(value!(false)));
        assert_eq!(evaluate("true == true"), Ok(value!(true)));
        assert_eq!(evaluate("\"value\" == \"value\""), Ok(value!(true)));
        assert_eq!(evaluate("1 + 2 == 3"), Ok(value!(true)));
    }

    #[test]
    fn decodes_string_escapes() {
        assert_eq!(
            evaluate(r#""line\n\t\"quote\"\\path""#),
            Ok(value!("line\n\t\"quote\"\\path"))
        );
        assert_eq!(evaluate(r#""unknown\q""#), Ok(value!(r"unknown\q")));
    }

    #[test]
    fn rejects_equality_for_unsupported_values() {
        assert_eq!(evaluate_error("[1] == [1]"), "type mismatch");
    }

    #[test]
    fn resolves_qualified_symbols_and_imports() {
        assert_eq!(
            evaluate("let std = import(\"std\")\nstd.math.PI"),
            Ok(value!(std::f64::consts::PI))
        );
        assert_eq!(
            evaluate("let std = import(\"std\")\nstd.math.sin(std.math.PI / 2)"),
            Ok(value!(1.0))
        );
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(evaluate_error("result: 1 / 0"), "division by zero");
    }

    #[test]
    fn resolves_chained_access_and_calls() {
        assert_eq!(
            evaluate("let types = import(\"types\")\n(types.int.sqrt)(9)"),
            Ok(value!(3.0))
        );
        assert_eq!(
            evaluate("result: 9.sqrt()"),
            Ok(Value::Map(map! { result: 3.0 }))
        );
    }

    #[test]
    fn expect_errors() {
        let cases = [
            ("division by zero", "result: 1 / 0"),
            ("integer overflow", "result: 9223372036854775807 + 1"),
            ("type mismatch", "result: true + 1"),
            ("unsupported expression", "result: unknown"),
            ("unknown function", "result: missing(1)"),
            ("not callable", "let value = 1\nresult: value(2)"),
            ("duplicate key", "result: 1\nresult: 2"),
            ("mixed document forms", "1\nresult: 2"),
            ("multiple expressions", "1\n2"),
        ];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let error = evaluate_error(input);
            fmt_snapshot_case(label, &[("input", input), ("error", &error)])
        });

        assert_snapshot!(cases);
    }

    #[test]
    fn expect_diagnostics() {
        let cases = [
            ("division by zero", "result: 1 / 0"),
            ("integer overflow", "result: 9223372036854775807 + 1"),
            ("type mismatch", "result: true + 1"),
            ("unknown identifier", "result: unknown"),
            ("unknown member", "result: {value: 1}.missing"),
            ("unknown function", "result: missing(1)"),
            ("unknown method", "result: 1.missing()"),
            ("not callable", "let value = 1\nresult: value(2)"),
            ("duplicate key", "result: 1\nresult: 2"),
            ("duplicate map key", "result: {value: 1, value: 2}"),
            ("pattern mismatch", "let [a, b] = [1]"),
            ("symbol conflict", "let value = 1\nlet value = 2"),
            ("mixed document forms", "1\nresult: 2"),
            ("multiple expressions", "1\n2"),
            (
                "assertion failed",
                "let std = import('std')\nstd.assert.assert(false)",
            ),
        ];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let error = evaluate(input).expect_err("expected an evaluation error");
            fmt_diagnostic_case(label, input, error)
        });

        assert_snapshot!(cases);
    }

    #[test]
    fn asserts_nested_entries_with_dotted_paths() {
        let document = map! {
            server: value!({ database: { host: "localhost" } }),
        };

        super::test_utils::assert_entries(
            &document,
            &[("server.database.host", value!("localhost"))],
        );
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    #[allow(clippy::missing_panics_doc)]
    pub fn assert_entries(document: &Document, expected: &[(&str, Value)]) {
        for (key, value) in expected {
            assert_eq!(document.get_path(key), Some(value));
        }
    }
}
