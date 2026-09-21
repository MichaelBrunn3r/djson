pub(crate) struct Parser<'src> {
    src: &'src [u8],
    pos: usize,
}

impl<'src> Parser<'src> {
    pub(crate) fn new(src: &'src [u8]) -> Self {
        Self { src, pos: 0 }
    }

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
        if self.peek() != Some(b']') {
            return false;
        }

        self.pos += 1;
        true
    }

    /// Consumes `byte` if it comes next, otherwise reports a missing `expected`.
    fn expect(&mut self, byte: u8, expected: &'static str) -> ParserResult<()> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(ParserError::Expected {
                pos: self.pos,
                expected,
            })
        }
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
}

//region ParserError
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ParserError {
    #[error("invalid integer")]
    InvalidInt {
        #[label("expected at least one digit")]
        pos: usize,
    },
    #[error("integer overflow for `{ty}`")]
    Overflow {
        #[label("does not fit in `{ty}`")]
        pos: usize,
        ty: &'static str,
    },
    #[error("expected {expected}")]
    Expected {
        #[label("expected {expected}")]
        pos: usize,
        expected: &'static str,
    },
    #[cfg(feature = "serde")]
    #[error("{msg}")]
    Custom { msg: String },
}

#[cfg(feature = "serde")]
impl serde::de::Error for ParserError {
    fn custom<T: std::fmt::Display>(message: T) -> Self {
        Self::Custom {
            msg: message.to_string(),
        }
    }
}

pub type ParserResult<T> = Result<T, ParserError>;
//endregion ParserError

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_u64() {
        for (src, expected) in [
            ("0", 0),
            ("1", 1),
            ("007", 7),
            ("12345", 12345),
            ("18446744073709551615", u64::MAX),
            // With separators
            ("1_", 1),
            ("1_2", 12),
            ("1__0", 10),
            ("1_000", 1_000),
            ("18_446_744_073_709_551_615", u64::MAX),
        ] {
            assert_parses(src, expected);
        }
    }

    #[test]
    fn rejects_input_that_does_not_start_with_a_digit() {
        for src in ["", "abc", "@", "+1", " 1", "_", "___", "_1"] {
            assert_rejected_without_advancing(src);
        }
    }

    #[test]
    fn rejects_values_above_u64_max() {
        let (result, _rest) = parse("18_446_744_073_709_551_616");
        assert!(
            matches!(result, Err(ParserError::Overflow { pos: 0, ty: "u64" })),
            "a value above `u64::MAX` should overflow `u64`"
        );
    }

    #[test]
    fn stops_at_the_first_byte_outside_the_number() {
        for (src, expected, rest) in [
            ("12,", 12, ","),
            ("1x", 1, "x"),
            ("1 2", 1, " 2"),
            ("1.5", 1, ".5"),
            ("1_000_000e", 1_000_000, "e"),
        ] {
            let (result, actual_rest) = parse(src);
            let value = result.unwrap_or_else(|error| panic!("parsing `{src}`: {error}"));
            assert_eq!(value, expected, "value of `{src}`");
            assert_eq!(actual_rest, rest, "unconsumed rest of `{src}`");
        }
    }

    //region Utils
    /// Parses `src` as a `u64` and returns the input left unconsumed.
    fn parse(src: &str) -> (ParserResult<u64>, &str) {
        let src_bytes = src.as_bytes();
        let mut parser = Parser::new(src_bytes);
        let result = parser.parse_u64();
        let rest = std::str::from_utf8(&src_bytes[parser.pos..]).expect("test input is text");
        (result, rest)
    }

    /// Asserts that `src` parses to `expected` and is consumed entirely.
    fn assert_parses(src: &str, expected: u64) {
        let (result, rest) = parse(src);
        let value = result.unwrap_or_else(|error| panic!("parsing `{src}`: {error}"));
        assert_eq!(value, expected, "value of `{src}`");
        assert_eq!(rest, "", "`{src}` is consumed entirely");
    }

    /// Asserts that nothing is consumed of `src` and it reports `error`.
    fn assert_rejected_without_advancing(src: &str) {
        let (result, rest) = parse(src);
        assert!(
            matches!(result, Err(ParserError::InvalidInt { .. })),
            "parsing `{src}` should report an invalid integer"
        );
        assert_eq!(rest, src, "nothing is consumed of `{src}`");
    }
    //endregion Utils
}
