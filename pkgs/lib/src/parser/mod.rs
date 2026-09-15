#![allow(clippy::missing_errors_doc)]

pub mod ast;
pub mod parse_expr;

use std::iter::Peekable;

use ast::AST;
use miette::SourceSpan;

use crate::{
    lexer::{Lexer, LexerError, token::Token},
    parser::ast::{Expr, Identifier, InfixOp, KV, Let, MapPattern, Pattern, Statement},
    span::Spanned,
};

//region Parser
pub struct Parser<'input> {
    tokens: Peekable<Lexer<'input>>,
    last_span: SourceSpan,
}

impl<'input> Parser<'input> {
    #[must_use]
    pub fn new(input: &'input str) -> Self {
        Self {
            tokens: Lexer::new(input).peekable(),
            last_span: (0, 0).into(),
        }
    }

    //region Parse statements
    /// Parses the complete input into an AST.
    pub fn parse_stmnts(mut self) -> ParserResult<AST<'input>> {
        let mut statements = Vec::new();
        self.skip_separators()?;

        while self.tokens.peek().is_some() {
            statements.push(self.parse_stmnt()?);
            self.skip_separators()?;
        }

        Ok(AST { statements })
    }

    /// Parses one statement, either a binding, key-value pair, or expression.
    fn parse_stmnt(&mut self) -> ParserResult<Spanned<Statement<'input>>> {
        let first = self.next_token()?;
        let start = first.span;

        if matches!(first.value, Token::Id("let")) && !self.next_is(&Token::Colon) {
            return self.parse_let_stmnt(start);
        }

        let expression = self.parse_expr(first, 0)?;
        if !self.next_is(&Token::Colon) {
            if matches!(
                self.tokens.peek(),
                Some(Ok(Spanned { value: token, .. })) if !matches!(token, Token::Sep)
            ) {
                let token = self.next_token()?;
                return Err(Self::err_unexpected(&token, "a separator or ':'"));
            }
            let span = expression.span;
            return Ok(Spanned {
                value: Statement::Expr(expression),
                span,
            });
        }

        let Spanned {
            value,
            span: key_span,
        } = expression;
        let (Expr::Id(Identifier::Simple(key)) | Expr::Str(key)) = value else {
            return Err(ParserError::InvalidKey { span: key_span });
        };

        self.next_token()?;
        let first = self.next_token()?;
        let expr = self.parse_expr(first, 0)?;
        let span = self.span_from(start);
        Ok(Spanned {
            value: Statement::KV(Spanned {
                value: KV {
                    key: Spanned {
                        value: key,
                        span: key_span,
                    },
                    expr,
                },
                span,
            }),
            span,
        })
    }

    /// Parses a `let` binding statement.
    fn parse_let_stmnt(&mut self, start: SourceSpan) -> ParserResult<Spanned<Statement<'input>>> {
        let pattern = self.parse_pattern()?;
        self.expect_next_token(&Token::Eq)?;
        let first = self.next_token()?;
        let expr = self.parse_expr(first, 0)?;
        let span = self.span_from(start);
        Ok(Spanned {
            value: Statement::Let(Spanned {
                value: Let { pattern, expr },
                span,
            }),
            span,
        })
    }
    //endregion Parse statements

    //region Parse pattern
    fn parse_pattern(&mut self) -> ParserResult<Spanned<Pattern<'input>>> {
        if self.next_is(&Token::LBrace) {
            let start = self.next_token()?.span;
            self.parse_pattern_body(start)
        } else if self.next_is(&Token::LBracket) {
            let start = self.next_token()?.span;
            self.parse_pattern_list(start)
        } else {
            let token = self.next_token()?;
            Self::parse_pattern_name(token)
        }
    }

    fn parse_pattern_list(&mut self, start: SourceSpan) -> ParserResult<Spanned<Pattern<'input>>> {
        let mut patterns = Vec::new();
        let mut rest = None;
        self.skip_separators()?;

        while !self.next_is(&Token::RBracket) {
            if self.next_is(&Token::DotDot) {
                self.next_token()?;
                rest = Some(match self.next_token()? {
                    Spanned {
                        value: Token::Id(value),
                        span,
                    } => Spanned { value, span },
                    token => return Err(Self::err_unexpected(&token, "an identifier")),
                });
                break;
            }

            patterns.push(self.parse_pattern()?);

            if !self.next_is(&Token::RBracket) {
                self.expect_next_token(&Token::Sep)?;
                self.skip_separators()?;
            }
        }

        self.expect_next_token(&Token::RBracket)?;
        Ok(self.spanned(start, Pattern::List { patterns, rest }))
    }

    fn parse_pattern_name(token: Spanned<Token<'input>>) -> ParserResult<Spanned<Pattern<'input>>> {
        let span = token.span;
        let name = match token.value {
            Token::Id(name) => name,
            value => {
                return Err(Self::err_unexpected(
                    &Spanned { value, span },
                    "an identifier or map pattern",
                ));
            }
        };
        Ok(Spanned {
            value: Pattern::Name(name),
            span,
        })
    }

    fn parse_pattern_body(&mut self, start: SourceSpan) -> ParserResult<Spanned<Pattern<'input>>> {
        let mut patterns = Vec::new();
        self.skip_separators()?;

        while !self.next_is(&Token::RBrace) {
            let token = self.next_token()?;
            let key_span = token.span;
            let Spanned {
                value: Token::Id(key),
                ..
            } = token
            else {
                return Err(Self::err_unexpected(
                    &Spanned {
                        value: Token::RBrace,
                        span: key_span,
                    },
                    "an identifier",
                ));
            };
            let (pattern, default) = if self.next_is(&Token::Eq) {
                self.next_token()?;
                let first = self.next_token()?;
                let default = self.parse_expr(first, 0)?;
                (self.spanned(key_span, Pattern::Name(key)), Some(default))
            } else if self.next_is(&Token::LBracket) {
                (self.parse_pattern()?, None)
            } else if self.next_is(&Token::Dot) {
                self.next_token()?;
                let body_start = self.expect_next_token(&Token::LBrace)?.span;
                (self.parse_pattern_body(body_start)?, None)
            } else {
                (self.spanned(key_span, Pattern::Name(key)), None)
            };
            let span = self.span_from(key_span);
            patterns.push(Spanned {
                value: MapPattern {
                    key,
                    pattern,
                    default,
                },
                span,
            });

            if !self.next_is(&Token::RBrace) {
                self.expect_next_token(&Token::Sep)?;
                self.skip_separators()?;
            }
        }

        self.expect_next_token(&Token::RBrace)?;
        Ok(self.spanned(start, Pattern::Map(patterns)))
    }
    //endregion Parse pattern

    fn next_is(&mut self, expected: &Token<'input>) -> bool {
        matches!(self.tokens.peek(), Some(Ok(token)) if &token.value == expected)
    }

    const fn infix_binding_power(token: &Token<'input>) -> Option<(u8, u8, InfixOp)> {
        match token {
            Token::Add => Some((10, 11, InfixOp::Add)),
            Token::Sub => Some((10, 11, InfixOp::Sub)),
            Token::Mul => Some((20, 21, InfixOp::Mul)),
            Token::Div => Some((20, 21, InfixOp::Div)),
            Token::Exp => Some((30, 30, InfixOp::Exp)),
            Token::Equal => Some((5, 6, InfixOp::Equal)),
            _ => None,
        }
    }

    fn skip_separators(&mut self) -> ParserResult<()> {
        while matches!(
            self.tokens.peek(),
            Some(Ok(Spanned {
                value: Token::Sep,
                ..
            }))
        ) {
            self.next_token()?;
        }
        Ok(())
    }

    fn expect_next_token(
        &mut self,
        expected: &Token<'input>,
    ) -> ParserResult<Spanned<Token<'input>>> {
        let token = self.next_token()?;
        if token.value == *expected {
            Ok(token)
        } else {
            Err(Self::err_unexpected(&token, &format!("{expected:?}")))
        }
    }

    fn next_token(&mut self) -> ParserResult<Spanned<Token<'input>>> {
        let token = self
            .tokens
            .next()
            .transpose()
            .map_err(ParserError::from)?
            .ok_or(ParserError::UnexpectedEof {
                span: self.last_span,
            })?;
        self.last_span = token.span;
        Ok(token)
    }

    fn err_unexpected(token: &Spanned<Token<'input>>, expected: &str) -> ParserError {
        ParserError::UnexpectedToken {
            expected: expected.to_owned(),
            found: format!("{:?}", token.value),
            span: token.span,
        }
    }

    /// The end offset of the most recently consumed token.
    const fn last_token_end(&self) -> usize {
        self.last_span.offset() + self.last_span.len()
    }

    /// Builds a span covering `start` through the most recently consumed token.
    fn span_from(&self, start: SourceSpan) -> SourceSpan {
        (
            start.offset(),
            self.last_token_end().saturating_sub(start.offset()),
        )
            .into()
    }

    /// Pairs `value` with the span covering `start` through the most recently
    /// consumed token.
    fn spanned<T>(&self, start: SourceSpan, value: T) -> Spanned<T> {
        Spanned {
            value,
            span: self.span_from(start),
        }
    }
}

//endregion Parser

//region ParserResult
pub type ParserResult<T> = Result<T, ParserError>;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ParserError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Lexer(#[from] LexerError),

    #[error("unexpected end of input")]
    #[diagnostic(code(parser::unexpected_eof))]
    UnexpectedEof {
        #[label("input ends here")]
        span: SourceSpan,
    },

    #[error("unexpected token")]
    #[diagnostic(code(parser::unexpected_token))]
    UnexpectedToken {
        expected: String,
        found: String,
        #[label("expected {expected}, got {found} instead")]
        span: SourceSpan,
    },

    #[error("invalid key")]
    #[diagnostic(code(parser::invalid_key))]
    InvalidKey {
        #[label("keys must be identifiers or strings")]
        span: SourceSpan,
    },
}
//endregion ParserResult

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::Parser;
    use crate::test_utils::{dedent, fmt_diagnostic_case, fmt_snapshot_case, fmt_snapshot_cases};

    #[test]
    fn expect_asts() {
        let cases = vec![
            (
                "key value pairs",
                "a: 1
                  b: 2
                  \"key with spaces\": 3",
            ),
            (
                "expression continuation after let binding",
                "let x = 1
                  x + 1",
            ),
            (
                "import expressions",
                "let std = import(\"std\")
                 std.math.sin(0)",
            ),
            (
                "keywords as top-level key",
                "let: 1
                 if: 2
                 else: 3",
            ),
            (
                "keywords as map key",
                "{
                    let: 1
                    if: 2
                    else: 3
                }",
            ),
            (
                "quoted expression-shaped top-level key",
                "\"not_a_string()\": 2",
            ),
            ("let binding with map", "let value = { nested: 7 }"),
            (
                "let binding followed by a document field",
                "let value = 7
                 result: value",
            ),
        ];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let input = dedent(input);
            let document = Parser::new(&input).parse_stmnts().expect("valid document");
            let input = input.replace('\n', "\n        ");
            let ast = document.pretty_string();
            format!("{label}\ninput: `{input}`\nast: {ast}")
        });

        assert_snapshot!(cases);
    }

    #[test]
    fn expect_errors() {
        let cases = [
            ("adjacent top-level tokens", "key key"),
            ("arithmetic expression as key", "1 + 1: 1"),
            ("call expression as key", "x(): 1"),
            ("member expression as key", "x.foo(): 1"),
            ("multi-word unquoted key", "let there be rain: 1"),
            ("non-identifier binding name", "let 1 = 1"),
        ];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let error = Parser::new(input)
                .parse_stmnts()
                .expect_err("expected a parser error");
            let error = error.to_string();
            fmt_snapshot_case(label, &[("input", input), ("error", &error)])
        });

        assert_snapshot!(cases);
    }

    #[test]
    fn parses_nested_destructuring_patterns() {
        let ast = Parser::new("let {a.{b, c.{d}}} = value")
            .parse_stmnts()
            .expect("valid destructuring pattern");

        assert_eq!(ast.pretty_string(), "[let {a.{b, c.{d}}} = value]");
    }

    #[test]
    fn parses_list_destructuring_patterns() {
        let ast = Parser::new("let [a, b] = value")
            .parse_stmnts()
            .expect("valid list destructuring pattern");

        assert_eq!(ast.pretty_string(), "[let [a, b] = value]");

        let ast = Parser::new("let {nested[a]} = value")
            .parse_stmnts()
            .expect("valid list pattern nested in a map pattern");

        assert_eq!(ast.pretty_string(), "[let {nested[a]} = value]");

        let ast = Parser::new("let [a, ..rest] = value")
            .parse_stmnts()
            .expect("valid list rest pattern");

        assert_eq!(ast.pretty_string(), "[let [a, ..rest] = value]");
    }

    #[test]
    fn parses_default_patterns() {
        let ast = Parser::new("let {x = 10} = value")
            .parse_stmnts()
            .expect("valid default pattern");

        assert_eq!(ast.pretty_string(), "[let {x = 10} = value]");

        let ast = Parser::new("let {a.{b.{c.{d.{e = 10}}}}} = value")
            .parse_stmnts()
            .expect("valid nested default pattern");

        assert_eq!(
            ast.pretty_string(),
            "[let {a.{b.{c.{d.{e = 10}}}}} = value]"
        );
    }

    #[test]
    fn allows_let_as_a_map_pattern_name() {
        let ast = Parser::new("let {let} = value")
            .parse_stmnts()
            .expect("valid destructuring pattern");

        assert_eq!(ast.pretty_string(), "[let {let} = value]");

        let ast = Parser::new("let {let.{let}} = value")
            .parse_stmnts()
            .expect("valid nested pattern");

        assert_eq!(ast.pretty_string(), "[let {let.{let}} = value]");

        let ast = Parser::new("let let = value")
            .parse_stmnts()
            .expect("valid name pattern");

        assert_eq!(ast.pretty_string(), "[let let = value]");
    }

    #[test]
    fn expect_diagnostics() {
        let cases = [
            (
                "lexer error",
                "{
                    value: 9223372036854775808
                }",
            ),
            (
                "unexpected end of input",
                "before: 1
                 value:",
            ),
            (
                "unexpected token",
                "before: 1
                  value: )
                 after: 3",
            ),
        ];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let input = &dedent(input);
            let error = Parser::new(input)
                .parse_stmnts()
                .expect_err("expected a parser error");
            fmt_diagnostic_case(label, input, error)
        });

        assert_snapshot!(cases);
    }
}
