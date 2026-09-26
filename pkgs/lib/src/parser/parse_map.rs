use super::Parser;
use super::error::{ParserError, ParserResult};

impl<'src> Parser<'src> {
    /// Consumes the start of a map.
    ///
    /// ## Errors
    /// The following bytes do not start a map
    pub(crate) fn expect_map_start(&mut self) -> ParserResult<()> {
        self.skip_whitespace();
        self.expect(b'{', "`{`")
    }

    /// Consumes the end of a map.
    pub(crate) fn eat_map_end(&mut self) -> bool {
        self.eat(b'}')
    }

    /// Consumes the separator between a key-value pair.
    pub(crate) fn expect_kv_separator(&mut self) -> ParserResult<()> {
        self.skip_whitespace();
        self.expect(b':', "`:`")?;
        self.skip_whitespace();
        Ok(())
    }

    /// Parses a quoted key and matches it against a known list.
    pub(crate) fn parse_known_key(
        &mut self,
        identifiers: &'static [&'static str],
    ) -> ParserResult<&'static str> {
        self.expect(b'"', "`\"`")?;

        let key_start = self.pos;
        let rest = &self.src[key_start..];

        // For one of the keys to match, `rest` must start with `key"`.
        if let Some(identifier) = identifiers.iter().copied().find(|identifier| {
            let bytes = identifier.as_bytes();
            rest.get(bytes.len()) == Some(&b'"') && rest.starts_with(bytes)
        }) {
            self.pos = key_start + identifier.len() + 1;
            return Ok(identifier);
        }

        // No match found -> Extract the offending identifier
        let key_len =
            rest.iter()
                .position(|&byte| byte == b'"')
                .ok_or(ParserError::UnterminatedString {
                    pos: self.src.len(),
                })?;
        Err(ParserError::InvalidIdentifier {
            pos: key_start,
            found: String::from_utf8_lossy(&rest[..key_len]).into_owned(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        from_bytes, parser::error::ParserErrorKind, utils::test::assert_deserializes_cases,
    };

    #[test]
    fn deserializes() {
        assert_deserializes_cases! {
            r#"{"a": [1,2,3], "b": {"a": 1, "b": "hi", "c": 3}}"# => nested(vec![1,2,3], three(1,"hi", 3)), // Nested

            // Partial
            "{}" => partial(None, None),
            r#"{"a": 1, "b": 2}"# => partial(Some(1), Some(2)),
            r#"{"a": 1}"# => partial(Some(1), None),
            r#"{"b": 1}"# => partial(None, Some(1)),
            r#"{"a": null "b": null}"# => partial(None, None),

            // Entry separators
            r#"{"a": 1,"b": "hi","c": 3}"# => three(1, "hi", 3),
            r#"{"a": 1 "b": "hi" "c": 3}"# => three(1, "hi", 3),
            "{\"a\": 1\n\"b\": \"hi\"\n\"c\": 3}" => three(1, "hi", 3),
            "{, ,\n,\r,\t,\"a\": 1, ,\n,\r,\t,\"b\": \"hi\", ,\n,\r,\t,\"c\": 3, ,\n,\r,\t,}" => three(1, "hi", 3),

            // Recognizes the correct known key, even if keys share prefixes
            r#"{"car": 1, "carpet": 2}"# => keys_prefix(1, 2),
            r#"{"carpet": 2, "car": 1}"# => keys_prefix(1, 2),
        }
    }

    #[test]
    fn expect_errors() {
        let cases = [
            (
                r#"{"a": 1"b": "hi","c": 3}"#,
                ParserErrorKind::MissingSeparator,
            ),
            (r#"{"x": 1, "b": 2}"#, ParserErrorKind::InvalidIdentifier),
            (r#"{"a: 1}"#, ParserErrorKind::UnterminatedString),
            ("{}", ParserErrorKind::Custom), // Missing keys
        ];

        for (src, expected) in cases {
            let actual = match from_bytes::<ThreeKeys>(src.as_bytes()) {
                Ok(_) => panic!("`{src}` was meant to fail"),
                Err(error) => error.kind(),
            };

            assert_eq!(actual, expected, "parsing `{src}`");
        }
    }

    //region Utils
    #[derive(Debug, serde_derive::Deserialize, PartialEq)]
    struct ThreeKeys {
        a: u32,
        b: String,
        c: u32,
    }

    fn three(a: u32, b: &str, c: u32) -> ThreeKeys {
        ThreeKeys { a, b: b.into(), c }
    }

    #[derive(Debug, serde_derive::Deserialize, PartialEq)]
    struct Nested {
        a: Vec<u32>,
        b: ThreeKeys,
    }

    fn nested(a: Vec<u32>, b: ThreeKeys) -> Nested {
        Nested { a, b }
    }

    #[derive(Debug, serde_derive::Deserialize, PartialEq)]
    struct Partial {
        a: Option<u32>,
        b: Option<u32>,
    }

    fn partial(a: Option<u32>, b: Option<u32>) -> Partial {
        Partial { a, b }
    }

    /// 2 keys, one is the prefix of the other
    #[derive(Debug, serde_derive::Deserialize, PartialEq)]
    struct KeysPrefix {
        car: u32,
        carpet: u32,
    }

    fn keys_prefix(car: u32, carpet: u32) -> KeysPrefix {
        KeysPrefix { car, carpet }
    }

    //endregion Utils
}
