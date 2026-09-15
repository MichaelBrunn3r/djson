use miette::SourceSpan;

use crate::{
    lexer::token::Token,
    parser::{
        Parser, ParserResult,
        ast::{Expr, Identifier, InfixOp, KV, PrefixOp},
    },
    span::Spanned,
};

impl<'input> Parser<'input> {
    /// Parses an expression.
    pub(super) fn parse_expr(
        &mut self,
        first: Spanned<Token<'input>>,
        min_bp: u8,
    ) -> ParserResult<Spanned<Expr<'input>>> {
        let start = first.span;
        let mut lhs = self.parse_lhs(first)?;

        while let Some(Ok(token)) = self.tokens.peek() {
            let Some((lhs_bp, rhs_bp, op)) = Self::infix_binding_power(&token.value) else {
                break;
            };

            if lhs_bp < min_bp {
                break;
            }

            self.next_token()?;
            lhs = self.parse_rhs_extension(lhs, rhs_bp, op)?;
        }

        Ok(self.spanned(start, lhs.value))
    }

    /// Parses the left-hand side of an expression.
    fn parse_lhs(&mut self, token: Spanned<Token<'input>>) -> ParserResult<Spanned<Expr<'input>>> {
        let span = token.span;
        match token.value {
            Token::Bool(value) => self.parse_postfix_op(span, Expr::Bool(value)),
            Token::Int(value) => self.parse_postfix_op(span, Expr::Int(value)),
            Token::Float(value) => self.parse_postfix_op(span, Expr::Float(value)),
            Token::Str(value) => self.parse_postfix_op(span, Expr::Str(value)),
            Token::LBracket => self.parse_list(span),
            Token::LBrace => self.parse_map(span),
            Token::Id("if") if !self.next_is(&Token::Colon) => self.parse_if(span),
            Token::Id(value) => self.parse_postfix_op(span, Expr::Id(Identifier::Simple(value))),
            Token::LParen => {
                let first = self.next_token()?;
                let expression = self.parse_expr(first, 0)?;
                self.expect_next_token(&Token::RParen)?;
                self.parse_postfix_op(self.span_from(span), expression.value)
            }
            Token::Add => self.parse_prefix_op(span, PrefixOp::Positive),
            Token::Sub => self.parse_prefix_op(span, PrefixOp::Negative),
            value => Err(Self::err_unexpected(&Spanned { value, span }, "expression")),
        }
    }

    fn parse_rhs_extension(
        &mut self,
        lhs: Spanned<Expr<'input>>,
        rhs_bp: u8,
        op: InfixOp,
    ) -> ParserResult<Spanned<Expr<'input>>> {
        let start = lhs.span;
        let first = self.next_token()?;
        let right = self.parse_expr(first, rhs_bp)?;
        Ok(self.spanned(
            start,
            Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(right),
            },
        ))
    }

    /// Parses a prefix operator and its operand.
    fn parse_prefix_op(
        &mut self,
        start: SourceSpan,
        op: PrefixOp,
    ) -> ParserResult<Spanned<Expr<'input>>> {
        let first = self.next_token()?;
        let value = self.parse_expr(first, 25)?;
        Ok(self.spanned(
            start,
            Expr::Unary {
                op,
                value: Box::new(value),
            },
        ))
    }

    fn parse_postfix_op(
        &mut self,
        start: SourceSpan,
        expr: Expr<'input>,
    ) -> ParserResult<Spanned<Expr<'input>>> {
        let mut expr = self.spanned(start, expr);

        loop {
            expr = match self.tokens.peek() {
                Some(Ok(Spanned {
                    value: Token::Dot, ..
                })) => {
                    self.next_token()?;
                    let name = match self.next_token()? {
                        Spanned {
                            value: Token::Id(value),
                            ..
                        } => value,
                        token => {
                            return Err(Self::err_unexpected(&token, "an identifier"));
                        }
                    };
                    self.spanned(
                        start,
                        Expr::Access {
                            object: Box::new(expr),
                            name,
                        },
                    )
                }
                Some(Ok(Spanned {
                    value: Token::LParen,
                    ..
                })) => self.parse_call(start, expr)?,
                _ => return Ok(expr),
            };
        }
    }

    /// Parses a function call and its arguments.
    fn parse_call(
        &mut self,
        start: SourceSpan,
        callee: Spanned<Expr<'input>>,
    ) -> ParserResult<Spanned<Expr<'input>>> {
        self.expect_next_token(&Token::LParen)?;
        let mut arguments = Vec::new();
        self.skip_separators()?;

        if !matches!(
            self.tokens.peek(),
            Some(Ok(Spanned {
                value: Token::RParen,
                ..
            }))
        ) {
            loop {
                let first = self.next_token()?;
                arguments.push(self.parse_expr(first, 0)?);

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RParen,
                        ..
                    }))
                ) {
                    break;
                }

                self.expect_next_token(&Token::Sep)?;
                self.skip_separators()?;

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RParen,
                        ..
                    }))
                ) {
                    break;
                }
            }
        }

        self.expect_next_token(&Token::RParen)?;
        Ok(self.spanned(
            start,
            Expr::Call {
                callee: Box::new(callee),
                arguments,
            },
        ))
    }

    /// Parses a list literal and its elements.
    fn parse_list(&mut self, start: SourceSpan) -> ParserResult<Spanned<Expr<'input>>> {
        let mut values = Vec::new();
        self.skip_separators()?;

        if !matches!(
            self.tokens.peek(),
            Some(Ok(Spanned {
                value: Token::RBracket,
                ..
            }))
        ) {
            loop {
                let first = self.next_token()?;
                values.push(self.parse_expr(first, 0)?);

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RBracket,
                        ..
                    }))
                ) {
                    break;
                }

                self.expect_next_token(&Token::Sep)?;
                self.skip_separators()?;

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RBracket,
                        ..
                    }))
                ) {
                    break;
                }
            }
        }

        self.expect_next_token(&Token::RBracket)?;
        self.parse_postfix_op(start, Expr::List(values))
    }

    /// Parses a map literal and its entries.
    fn parse_map(&mut self, start: SourceSpan) -> ParserResult<Spanned<Expr<'input>>> {
        let mut entries = Vec::new();
        self.skip_separators()?;

        if !matches!(
            self.tokens.peek(),
            Some(Ok(Spanned {
                value: Token::RBrace,
                ..
            }))
        ) {
            loop {
                let token = self.next_token()?;
                let key_span = token.span;
                let key = match token.value {
                    Token::Id(value) | Token::Str(value) => value,
                    value => {
                        return Err(Self::err_unexpected(
                            &Spanned {
                                value,
                                span: key_span,
                            },
                            "a map key",
                        ));
                    }
                };
                self.expect_next_token(&Token::Colon)?;
                let first = self.next_token()?;
                let expr = self.parse_expr(first, 0)?;
                entries.push(self.spanned(
                    key_span,
                    KV {
                        key: Spanned {
                            value: key,
                            span: key_span,
                        },
                        expr,
                    },
                ));

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RBrace,
                        ..
                    }))
                ) {
                    break;
                }

                self.expect_next_token(&Token::Sep)?;
                self.skip_separators()?;

                if matches!(
                    self.tokens.peek(),
                    Some(Ok(Spanned {
                        value: Token::RBrace,
                        ..
                    }))
                ) {
                    break;
                }
            }
        }

        self.expect_next_token(&Token::RBrace)?;
        self.parse_postfix_op(start, Expr::Map(entries))
    }

    /// Parses a conditional expression with `then` and `else` branches.
    fn parse_if(&mut self, start: SourceSpan) -> ParserResult<Spanned<Expr<'input>>> {
        let condition = {
            let first = self.next_token()?;
            self.parse_expr(first, 0)?
        };

        let then = {
            // Skip {
            self.expect_next_token(&Token::LBrace)?;
            self.skip_separators()?;

            let first = self.next_token()?;
            let expr = self.parse_expr(first, 0)?;

            // Skip }
            self.skip_separators()?;
            self.expect_next_token(&Token::RBrace)?;

            expr
        };

        let r#else = {
            // Skip 'else'
            match self.next_token()? {
                Spanned {
                    value: Token::Id("else"),
                    ..
                } => {}
                token => return Err(Self::err_unexpected(&token, "`else`")),
            }

            // Skip {
            self.expect_next_token(&Token::LBrace)?;
            self.skip_separators()?;

            let first = self.next_token()?;
            let expr = self.parse_expr(first, 0)?;

            // Skip }
            self.skip_separators()?;
            self.expect_next_token(&Token::RBrace)?;

            expr
        };

        self.parse_postfix_op(
            start,
            Expr::If {
                condition: Box::new(condition),
                then: Box::new(then),
                r#else: Box::new(r#else),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::Parser;
    use crate::test_utils::{dedent, fmt_snapshot_case, fmt_snapshot_cases};

    #[test]
    fn expect_asts() {
        let cases = vec![
            ("expression statements", "1 + 2 * 3"),
            ("string expression", "\"hello\""),
            (
                "math expressions",
                "sum: 1 + 2
                 difference: 5 - 2
                 product: 2 * 3
                 quotient: 8 / 2
                 precedence: 1 + 2 * 3
                 power: 2 ^ 3 ^ 4
                 positive: +1
                 negative: -2 ^ 2",
            ),
            (
                "function calls",
                "inline: foo(1, 2,3)
                 multiline: bar(
                    1
                    2,
                    3
                 )",
            ),
            ("conditional expression", "if (true) { 1 } else { 2 }"),
            ("member call expression", "x.foo()"),
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
        let cases = [("empty expression", "()")];

        let cases = fmt_snapshot_cases(cases, |(label, input)| {
            let error = Parser::new(input)
                .parse_stmnts()
                .expect_err("expected a parser error");
            let error = error.to_string();
            fmt_snapshot_case(label, &[("input", input), ("error", &error)])
        });

        assert_snapshot!(cases);
    }
}
