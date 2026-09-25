use crate::{ParserResult, parser::Parser};

impl<'src> Parser<'src> {
    /// Consumes the start of a list.
    ///
    /// ## Errors
    /// The following bytes do not start a list
    pub(crate) fn expect_list_start(&mut self) -> ParserResult<()> {
        self.skip_whitespace();
        self.expect(b'[', "`[`")
    }

    /// Consumes the end of a list.
    #[inline]
    pub(crate) fn eat_list_end(&mut self) -> bool {
        self.eat(b']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        from_bytes,
        parser::error::ParserErrorKind,
        utils::test::{
            assert_deserializes_cases, discard_ok, fmt_report, fmt_snapshot_case,
            fmt_snapshot_cases,
        },
    };

    #[test]
    fn deserializes() {
        assert_deserializes_cases! {
            "[]" => Vec::<u32>::new(), // Empty
            "[[1, 2], [], [3]]" => vec![vec![1u8, 2], vec![], vec![3]], // Nested
            r#"["a", "b\", c"]"# => vec!["a".to_owned(), "b\", c".into()],

            // Item separators
            "[1,2,3]" => vec![1u32, 2, 3],
            "[ 1 2 3 ]" => vec![1u32, 2, 3],
            "[1\n2\n3]" => vec![1u32, 2, 3],
            "[, ,\n,\r,\t,1, ,\n,\r,\t,2, ,\n,\r,\t,3, ,\n,\r,\t,]" => vec![1u32, 2, 3],
        }
    }

    #[test]
    fn expect_errors() {
        let cases = [(r#"["a""b"]"#, ParserErrorKind::MissingSeparator)];

        for (src, expected) in cases {
            let actual = match from_bytes::<Vec<String>>(src.as_bytes()) {
                Ok(_) => panic!("`{src}` was meant to fail"),
                Err(error) => error.kind(),
            };

            assert_eq!(actual, expected, "parsing `{src}`");
        }
    }

    #[test]
    fn diagnostics() {
        let cases: &[(&str, &str, fn(&[u8]) -> ParserResult<()>)] = &[
            ("not a list", "1", discard_ok!(from_bytes::<Vec<u8>>)),
            (
                "missing separator",
                r#"["a""b"]"#,
                discard_ok!(from_bytes::<Vec<String>>),
            ),
            (
                "unterminated list",
                "[1, 2",
                discard_ok!(from_bytes::<Vec<u8>>),
            ),
        ];

        let snap = fmt_snapshot_cases(cases, |&(label, source_code, parse)| {
            let error = match parse(source_code.as_bytes()) {
                Ok(()) => panic!("case `{label}` was meant to fail: {source_code:?}"),
                Err(error) => error,
            };
            let report = miette::Report::from(error).with_source_code(source_code);
            fmt_snapshot_case(label, &[("error", &fmt_report(&report))])
        });
        insta::assert_snapshot!(snap);
    }
}
