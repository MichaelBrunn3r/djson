use self::error::{ParserError, ParserResult};

pub(crate) mod error;
mod parse_list;
mod parse_map;
mod parse_str;
mod parse_uint;
mod parse_utf16_escape;

pub(crate) use parse_str::ParsedStr;

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

    pub(crate) fn parse_bool(&mut self) -> ParserResult<bool> {
        if self.eat_keyword(b"true") {
            return Ok(true);
        } else if self.eat_keyword(b"false") {
            return Ok(false);
        }

        Err(ParserError::Expected {
            pos: self.pos,
            expected: "`true` or `false`",
        })
    }

    pub(crate) fn skip_entry_separator(&mut self) -> bool {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_whitespace() || byte == b',')
        {
            self.pos += 1;
        }

        self.pos != start
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

    /// Consumes the next byte if it matches `expected`.
    ///
    /// ## Returns
    /// Whether the byte was consumed.
    fn eat(&mut self, expected: u8) -> bool {
        if self.src.get(self.pos) != Some(&expected) {
            return false;
        }
        self.pos += 1;
        true
    }

    /// Consumes the next `n = expected.len()` bytes if they match `expected`.
    ///
    /// ## Returns
    /// Whether the bytes were consumed.
    fn eat_bytes(&mut self, expected: &[u8]) -> bool {
        let Some(src_rest) = self.src.get(self.pos..) else {
            return false;
        };

        if !src_rest.starts_with(expected) {
            return false;
        }

        self.pos += expected.len();
        true
    }

    /// Consumes the next `n = keyword.len()` bytes if they match `keyword` and
    /// are followed by an identifier boundary.
    ///
    /// ## Returns
    /// Whether the keyword was consumed.
    pub(crate) fn eat_keyword(&mut self, keyword: &[u8]) -> bool {
        let Some(src_rest) = self.src.get(self.pos..) else {
            return false;
        };

        if !src_rest.starts_with(keyword) {
            return false;
        }

        // Keyword must be a standalone token (e.g. `null 1`, `null]` but not `null_1`)
        if src_rest
            .get(keyword.len())
            .is_some_and(|&byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return false;
        }

        self.pos += keyword.len();
        true
    }

    /// Consumes the next byte if it matches `expected`.
    ///
    /// ## Errors
    fn expect(&mut self, byte: u8, expected: &'static str) -> ParserResult<()> {
        if self.eat(byte) {
            return Ok(());
        }

        Err(ParserError::Expected {
            pos: self.pos,
            expected,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::assert_deserializes_cases;

    #[test]
    fn eat_keyword() {
        assert_deserializes_cases! {
            "null" => None::<u32>,
            "[null]" => vec![None::<u32>],
            "[null,null]" => vec![None::<u32>, None::<u32>],
            "[ null , null ]" => vec![None::<u32>, None::<u32>],
        }
    }

    #[test]
    fn deserializes_options() {
        assert_deserializes_cases! {
            "1" => Some(1u32),
            r#""hi""# => Some("hi".to_owned()),

            // In container
            "[null, 1]" => vec![None::<u32>, Some(1u32)],
            "[[null], null]" => vec![Some(vec![None::<u32>]), None], // Nested
        }
    }

    #[test]
    fn deserializes_bools() {
        assert_deserializes_cases! {
            "true" => true,
            "false" => false,
            "[true, false]" => vec![true, false]
        }
    }

    // TRUE FALSE trUe FalSe yes no
}
