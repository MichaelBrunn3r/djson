use self::error::{ParserError, ParserResult};

pub(crate) mod error;
mod parse_str;
mod parse_utf16_escape;

pub(crate) struct Parser<'src> {
    pub(crate) src: &'src [u8],
    pub(crate) pos: usize,
    /// Scratch buffer for string parsing. Enables exact string allocations
    pub(crate) scratch: String,
}

impl<'src> Parser<'src> {
    pub(crate) fn new(src: &'src [u8]) -> Self {
        Self {
            src,
            pos: 0,
            scratch: String::new(),
        }
    }

    //region Parse uint

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

    //endregion Parse uint

    //region Parse list

    /// Consumes the `[` that opens a list.
    pub(crate) fn parse_list_start(&mut self) -> ParserResult<()> {
        self.skip_whitespace();
        self.expect(b'[', "`[`")
    }

    /// Advances to the next item of a list, consuming the `,` that separates it
    /// from the previous item.
    ///
    /// Pass `first` while no item has been visited yet. Returns `false` once the
    /// closing `]` has been consumed, which means the list is complete.
    pub(crate) fn parse_list_item(&mut self, first: bool) -> ParserResult<bool> {
        self.skip_whitespace();
        if !first {
            // A closing `]` ends the list without a preceding separator.
            if self.parse_list_end() {
                return Ok(false);
            }
            self.expect(b',', "`,` or `]`")?;
            self.skip_whitespace();
        }

        // A separator right before the closing `]` is a trailing comma.
        if self.parse_list_end() {
            return Ok(false);
        }

        if self.peek().is_none() {
            return Err(ParserError::Expected {
                pos: self.pos,
                expected: "`]`",
            });
        }

        Ok(true)
    }

    /// Consumes the `]` that closes a list if it comes next.
    fn parse_list_end(&mut self) -> bool {
        self.expect_next(b']')
    }

    //endregion Parse list

    /// Consumes `byte` if it comes next, otherwise reports a missing `expected`.
    fn expect(&mut self, byte: u8, expected: &'static str) -> ParserResult<()> {
        if self.expect_next(byte) {
            return Ok(());
        }

        Err(ParserError::Expected {
            pos: self.pos,
            expected,
        })
    }

    /// Skips the spaces, tabs and line breaks that are insignificant between
    /// tokens.
    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    /// Returns the next byte without consuming it.
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let byte = self.src.get(self.pos).copied()?;
        self.pos += 1;
        Some(byte)
    }

    /// Consumes `expected` if it comes next, otherwise noop and returns `false`.
    fn expect_next(&mut self, expected: u8) -> bool {
        if self.src.get(self.pos).copied() != Some(expected) {
            return false;
        }
        self.pos += 1;
        true
    }

    /// Consumes `expected` if `self.src` continues with exactly those bytes,
    /// otherwise noop and returns `false`
    fn expect_next_bytes(&mut self, expected: &[u8]) -> bool {
        let Some(rest) = self.src.get(self.pos..) else {
            return false;
        };

        if !rest.starts_with(expected) {
            return false;
        }

        self.pos += expected.len();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        from_bytes, parser::error::ParserErrorKind, test_utils::assert_deserializes_cases,
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
    fn fails_parsing_with_error() {
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
    fn deserializes_lists() {
        assert_deserializes_cases! {
            "[1, 2, 3]" => vec![1u32, 2, 3],
            "[[1, 2], [], [3]]" => vec![vec![1u8, 2], vec![], vec![3]], // Nested
            "[\n \t 1  , \n    \r2,\n]" => vec![1u8, 2], // Whitespace
            r#"["a", "b\", c"]"# => vec![String::from("a"), String::from("b\", c")],
        }
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
