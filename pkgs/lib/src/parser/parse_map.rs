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
    fn eat_map_end(&mut self) -> bool {
        self.eat(b'}')
    }

    /// Advances to the next key-value pair of a map
    pub(crate) fn advance_to_next_kv(&mut self, is_first: bool) -> ParserResult<bool> {
        self.skip_whitespace();
        if !is_first {
            if self.eat_map_end() {
                return Ok(false);
            }
            self.expect(b',', "`,` or `}`")?;
            self.skip_whitespace();
        }

        if self.eat_map_end() {
            return Ok(false);
        }

        if self.peek().is_none() {
            return Err(ParserError::Expected {
                pos: self.pos,
                expected: "`}`",
            });
        }

        Ok(true)
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
        self.skip_whitespace();
        self.expect(b'"', "`\"`")?;

        // Get the key
        let key_start = self.pos;
        let key_len = self.src[key_start..]
            .iter()
            .position(|&byte| byte == b'"')
            .ok_or(ParserError::UnterminatedString {
                pos: self.src.len(),
            })?;
        let key_end = key_start + key_len;
        let key = &self.src[key_start..key_end];

        // Match against allowed keys
        let Some(identifier) = identifiers
            .iter()
            .find(|identifier| identifier.as_bytes() == key)
            .copied()
        else {
            return Err(ParserError::InvalidIdentifier {
                pos: key_start,
                found: String::from_utf8_lossy(key).into_owned(),
            });
        };
        self.pos = key_end + 1;
        Ok(identifier)
    }
}

#[cfg(test)]
mod test {
    use crate::from_bytes;

    #[derive(Debug, serde_derive::Deserialize, PartialEq)]
    struct TestStruct {
        key: u32,
    }

    #[test]
    fn ignores_ws() {
        let parts = r#"{"key": 42}"#.split(" ").collect::<Vec<_>>();

        for ws_after_part_i in 0..parts.len() - 1 {
            let mut source = String::new();
            for (i, part) in parts.iter().enumerate() {
                source.push_str(part);
                if i == ws_after_part_i {
                    source.push_str(" \n\t ");
                }
            }

            let value: TestStruct = from_bytes(source.as_bytes()).expect("struct deserializes");
            assert_eq!(value, TestStruct { key: 42 }, "source: {source:?}");
        }
    }
}
