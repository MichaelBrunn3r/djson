use crate::{
    ParserResult,
    parser::{Parser, error::ParserError},
};

impl<'src> Parser<'src> {
    /// Parses a u64
    ///
    /// Grammar:
    /// ```text
    /// u64     = digit (sep | digit)*
    /// sep     = '_'
    /// digit   = '0' | ... | '9'
    /// ```
    fn parse_u64(&mut self) -> ParserResult<u64> {
        let start = self.pos;

        // Must start with a digit
        if !self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            return Err(ParserError::InvalidInt { pos: start });
        }

        let mut value: u64 = 0;

        //  18_446_744_073_709_551_615 - u64::MAX
        //   9_999_999_999_999_999_999 - the largest 19-digit number
        for &byte in self.src[start..].iter().take(19) {
            if !byte.is_ascii_digit() {
                break;
            }
            self.pos += 1;
            value = value * 10 + (byte - b'0') as u64;
        }

        // Slow path: 20th digit and beyond, separators, ...
        while let Some(byte) = self
            .peek()
            .filter(|&byte| matches!(byte, b'_' | b'0'..=b'9'))
        {
            self.pos += 1;
            if let b'0'..=b'9' = byte {
                value = value
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(u64::from(byte - b'0')))
                    .ok_or(ParserError::Overflow {
                        pos: start,
                        ty: "u64",
                    })?;
            }
        }

        Ok(value)
    }

    pub(crate) fn parse_uint<T>(&mut self) -> ParserResult<T>
    where
        T: TryFrom<u64>,
    {
        let start = self.pos;
        let value = self.parse_u64()?;
        T::try_from(value).map_err(|_| ParserError::Overflow {
            pos: start,
            ty: std::any::type_name::<T>(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ParserResult, from_bytes,
        parser::error::ParserErrorKind,
        utils::test::{
            assert_deserializes_cases, discard_ok, fmt_report, fmt_snapshot_case,
            fmt_snapshot_cases,
        },
    };

    #[test]
    fn deserializes_u64() {
        assert_deserializes_cases!(
            "0" => 0u64,
            "1" => 1u64,
            "007" => 7u64,
            "12345" => 12345u64,

            // With Separators
            "1_" => 1u64,
            "1_2" =>12u64,
            "1__0" => 10u64,
            "1_000" => 1_000u64,

            // Max value
            "255" => u8::MAX,
            "65535" => u16::MAX,
            "65_535" => u16::MAX,
            "4294967295" => u32::MAX,
            "4_294_967_295" => u32::MAX,
            "18446744073709551615" => u64::MAX,
            "18_446_744_073_709_551_615" => u64::MAX,
        );
    }

    #[test]
    fn expect_errors() {
        let cases: &[(&str, fn(&str) -> ParserResult<()>, ParserErrorKind)] = &[
            ("_1", parse_u64, ParserErrorKind::InvalidInt), // Leading separator
            ("__", parse_u64, ParserErrorKind::InvalidInt), // Only separators
            ("-1", parse_u64, ParserErrorKind::InvalidInt), // Negative
            ("+1", parse_u64, ParserErrorKind::InvalidInt), // Plus sign
            (" 1", parse_u64, ParserErrorKind::InvalidInt), // Leading whitespace
            //region Not a number
            ("", parse_u64, ParserErrorKind::InvalidInt),
            ("abc", parse_u64, ParserErrorKind::InvalidInt),
            (r#"["abc"]"#, parse_u64, ParserErrorKind::InvalidInt),
            //endregion Not a number
            //region Overflow
            ("256", from_bytes_uint::<u8>, ParserErrorKind::Overflow), // u8::MAX + 1
            ("65536", from_bytes_uint::<u16>, ParserErrorKind::Overflow), // u16::MAX + 1
            (
                // u32::MAX + 1
                "4294967296",
                from_bytes_uint::<u32>,
                ParserErrorKind::Overflow,
            ),
            (
                // u64::MAX + 1
                "18446744073709551616",
                from_bytes_uint::<u64>,
                ParserErrorKind::Overflow,
            ),
            //endregion Overflow
        ];

        for &(source, parse, expected) in cases {
            let actual = match parse(source) {
                Ok(()) => panic!("`{source}` was meant to fail"),
                Err(error) => error.kind(),
            };

            assert_eq!(actual, expected, "parsing `{source}`");
        }
    }

    #[test]
    fn diagnostics() {
        let cases: &[(&str, &str, fn(&[u8]) -> ParserResult<()>)] = &[
            ("overflows u8", "256", discard_ok!(from_bytes::<u8>)),
            ("overflows u16`", "65536", discard_ok!(from_bytes::<u16>)),
            ("no digits at all", "", discard_ok!(from_bytes::<u8>)),
            ("not a digit run", "abc", discard_ok!(from_bytes::<u8>)),
        ];

        let snap = fmt_snapshot_cases(cases, |&(label, source_code, parse)| {
            let error = parse(source_code.as_bytes())
                .err()
                .expect("case is meant to fail");
            let report = miette::Report::from(error).with_source_code(source_code);
            fmt_snapshot_case(label, &[("error", &fmt_report(&report))])
        });
        insta::assert_snapshot!(snap);
    }

    //region Utils
    fn parse_u64(src: &str) -> ParserResult<()> {
        Parser::new(src.as_bytes()).parse_u64().map(drop)
    }

    /// Reads `src` as `T` through the serde path.
    fn from_bytes_uint<T>(src: &str) -> ParserResult<()>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        from_bytes::<T>(src.as_bytes()).map(drop)
    }
    //endregion Utils
}
