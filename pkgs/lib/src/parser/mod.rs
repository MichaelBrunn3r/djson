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
