use crate::{
    ParserResult,
    parser::{Parser, error::ParserError},
};

impl<'src> Parser<'src> {
    /// Parses a signed integer.
    ///
    /// Grammar:
    /// ```text
    /// sint    = ['-'] u64
    /// ```
    pub(crate) fn parse_sint<T>(&mut self) -> ParserResult<T>
    where
        T: TryFrom<i128>,
    {
        let start = self.pos;
        let negative = self.eat(b'-');

        let magnitude = match self.parse_u64() {
            Ok(magnitude) => magnitude,
            Err(ParserError::Overflow { .. }) => {
                return Err(ParserError::Overflow {
                    pos: start,
                    ty: std::any::type_name::<T>(),
                });
            }
            Err(other) => return Err(other),
        };

        let value = if negative {
            -i128::from(magnitude)
        } else {
            i128::from(magnitude)
        };

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
        from_bytes, parser::error::ParserErrorKind, utils::test::assert_deserializes_cases,
    };

    #[test]
    fn deserializes_i64() {
        assert_deserializes_cases!(
            "0" => 0i64,
            "-0" => 0i64,
            "1" => 1i64,
            "-1" => -1i64,
            "007" => 7i64,
            "-007" => -7i64,
            "12345" => 12345i64,
            "-12345" => -12345i64,

            // With Separators
            "1_" => 1i64,
            "-1_2" => -12i64,
            "-1__0" => -10i64,
            "-1_000" => -1_000i64,

            // Bounds
            "127" => i8::MAX,
            "-128" => i8::MIN,
            "32_767" => i16::MAX,
            "-32_768" => i16::MIN,
            "2_147_483_647" => i32::MAX,
            "-2_147_483_648" => i32::MIN,
            "9223372036854775807" => i64::MAX,
            "9_223_372_036_854_775_807" => i64::MAX,
            "-9223372036854775808" => i64::MIN,
            "-9_223_372_036_854_775_808" => i64::MIN,
        );
    }

    #[test]
    fn expect_errors() {
        let cases: &[(&str, fn(&str) -> ParserResult<()>, ParserErrorKind)] = &[
            //region Not a number
            ("-", parse_i64, ParserErrorKind::InvalidInt), // Sign only
            ("--1", parse_i64, ParserErrorKind::InvalidInt), // Double sign
            ("+1", parse_i64, ParserErrorKind::InvalidInt), // Plus sign
            (" -1", parse_i64, ParserErrorKind::InvalidInt), // Leading whitespace
            ("", parse_i64, ParserErrorKind::InvalidInt),
            ("abc", parse_i64, ParserErrorKind::InvalidInt),
            //endregion Not a number
            //region Overflow
            ("128", from_bytes_sint::<i8>, ParserErrorKind::Overflow), // i8::MAX + 1
            ("-129", from_bytes_sint::<i8>, ParserErrorKind::Overflow), // i8::MIN - 1
            ("32768", from_bytes_sint::<i16>, ParserErrorKind::Overflow), // i16::MAX + 1
            ("-32769", from_bytes_sint::<i16>, ParserErrorKind::Overflow), // i16::MIN - 1
            (
                "2147483648", // i32::MAX + 1
                from_bytes_sint::<i32>,
                ParserErrorKind::Overflow,
            ),
            (
                "-2147483649", // i32::MIN - 1
                from_bytes_sint::<i32>,
                ParserErrorKind::Overflow,
            ),
            (
                "9223372036854775808", // i64::MAX + 1
                from_bytes_sint::<i64>,
                ParserErrorKind::Overflow,
            ),
            (
                "-9223372036854775809", // i64::MIN - 1
                from_bytes_sint::<i64>,
                ParserErrorKind::Overflow,
            ),
            (
                // u64::MAX + 1, overflows the magnitude parsing
                "18446744073709551616",
                from_bytes_sint::<i64>,
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

    //region Utils
    fn parse_i64(src: &str) -> ParserResult<()> {
        Parser::new(src.as_bytes()).parse_sint::<i64>().map(drop)
    }

    /// Reads `src` as `T` through the serde path.
    fn from_bytes_sint<T>(src: &str) -> ParserResult<()>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        from_bytes::<T>(src.as_bytes()).map(drop)
    }
    //endregion Utils
}
