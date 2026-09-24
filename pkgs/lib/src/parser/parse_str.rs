use std::borrow::Cow;

use super::Parser;
use super::error::{ParserError, ParserResult};
use crate::utils::U8Ext;

impl<'src> Parser<'src> {
    /// Parses a string.
    ///
    /// Accepts all valid [RFC 8259 (Json)](https://www.rfc-editor.org/rfc/rfc8259#section-7)
    /// strings, except broken UTF-16 escape pairs.
    ///
    /// ## Grammar:
    /// ```text
    /// string           = '"' char* '"'
    /// char             = unescaped | escape
    /// unescaped        = any character except '"', '\' and the C0 control characters
    /// escape           = utf16_code_point | '\' ( '"' | '\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')
    /// utf16_code_point = utf16_code_unit | utf16_code_unit utf16_code_unit
    /// utf16_code_unit  = '\' 'u' hex hex hex hex
    /// hex              = '0'-'9' | 'a'-'f' | 'A'-'F'
    /// ```
    ///
    /// ## Returns
    /// An owned String when the string contains escapes, a borrowed str otherwise.
    ///
    /// ## Errors
    /// - String contains invalid utf8
    /// - String contains an invalid UTF-16 escape
    #[inline]
    pub(crate) fn parse_str(&mut self) -> ParserResult<Cow<'src, str>> {
        let src = self.src;
        self.expect(b'"', "`\"`")?;
        let content_start = self.pos;

        let (chunk_len, chunk_is_ascii) = find_raw_chunk_end(&src[content_start..])
            .ok_or(ParserError::UnterminatedString { pos: src.len() })?;
        let content_end = content_start + chunk_len;

        match src[content_end] {
            // string is a single raw chunk (no escapes) -> return borrowed
            b'"' => {
                // unicode needs decoding, plain ascii doesn't
                if chunk_is_ascii {
                    self.pos = content_end + '"'.len_utf8();
                    // SAFETY: String is entirely ascii -> valid utf8
                    return Ok(Cow::Borrowed(unsafe {
                        std::str::from_utf8_unchecked(&src[content_start..content_end])
                    }));
                } else {
                    self.pos = content_end + '"'.len_utf8();
                    Ok(Cow::Borrowed(Self::decode_utf8(
                        &src[content_start..content_end],
                        content_start,
                    )?))
                }
            }
            // at least 1 escape -> allocate & decode
            b'\\' => self.parse_escape_chunks(content_start, content_end, chunk_is_ascii),
            // unescaped control char
            byte => Err(ParserError::ControlCharacter {
                pos: content_end,
                byte,
            }),
        }
    }

    /// Parses a string as a sequence of chunks separated by escape sequences.
    ///
    /// # Arguments
    /// - `content_start`       : Position after the opening `"`
    /// - `pos_backslash`       : The start of the first `\` in the string
    /// - `first_chunk_is_ascii`: Whether the first raw chunk is entirely ascii.
    ///
    /// # Contract
    /// Post: `self.pos` is after the closing `"`
    fn parse_escape_chunks(
        &mut self,
        content_start: usize,
        pos_backslash: usize,
        first_chunk_is_ascii: bool,
    ) -> ParserResult<Cow<'src, str>> {
        self.scratch.clear();

        let mut chunk_start = content_start;
        let mut chunk_end = pos_backslash; // pos of the char that ends the current chunk
        let mut chunk_is_ascii = first_chunk_is_ascii;
        loop {
            // Decode & push the finished chunk
            self.scratch_raw_chunk(
                &self.src[chunk_start..chunk_end],
                chunk_is_ascii,
                chunk_start,
            )?;

            // Parse the escape sequence
            self.pos = chunk_end + '\\'.len_utf8();
            let escaped = self.parse_escaped_char()?;
            self.scratch.push(escaped);
            chunk_start = self.pos;

            // Find the end of the next chunk
            let (next_chunk_len, next_is_ascii) = find_raw_chunk_end(&self.src[self.pos..]).ok_or(
                ParserError::UnterminatedString {
                    pos: self.src.len(),
                },
            )?;
            chunk_end = self.pos + next_chunk_len;
            chunk_is_ascii = next_is_ascii;

            match self.src[chunk_end] {
                // found an escape sequence
                b'\\' => {}
                // string terminated & no further escapes found
                b'"' => {
                    self.scratch_raw_chunk(
                        &self.src[chunk_start..chunk_end],
                        chunk_is_ascii,
                        chunk_start,
                    )?;
                    self.pos = chunk_end + '"'.len_utf8();
                    return Ok(Cow::Owned(String::from(self.scratch.as_str())));
                }
                byte => {
                    return Err(ParserError::ControlCharacter {
                        pos: chunk_end,
                        byte,
                    });
                }
            }
        }
    }

    /// Appends a raw chunk to the scratch buffer.
    ///
    /// # Arguments
    /// - `chunk`       : A raw chunk of the string content.
    /// - `is_ascii`    : Whether `chunk` is entirely ascii.
    /// - `chunk_start` : Starting position of `chunk` in the source.
    #[inline]
    fn scratch_raw_chunk(
        &mut self,
        chunk: &[u8],
        is_ascii: bool,
        chunk_start: usize,
    ) -> ParserResult<()> {
        let chunk = if is_ascii {
            // SAFETY: The entire chunk is ascii -> valid utf8
            unsafe { std::str::from_utf8_unchecked(chunk) }
        } else {
            Self::decode_utf8(chunk, chunk_start)?
        };

        self.scratch.push_str(chunk);
        Ok(())
    }

    /// Decodes an escape sequence to a char.
    ///
    /// # Contract
    /// Pre: `self.pos` is on the byte behind `\`
    fn parse_escaped_char(&mut self) -> ParserResult<char> {
        debug_assert_eq!(self.src.get(self.pos - 1), Some(&b'\\'));

        let start = self.pos;
        let Some(byte) = self.next() else {
            return Err(ParserError::UnterminatedString { pos: start });
        };

        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_escaped_utf16_char(start),
            _ => Err(ParserError::InvalidEscape {
                pos: start,
                msg: "unknown escape sequence",
            }),
        }
    }

    /// Decodes `bytes` as UTF-8, pointing at `pos` when it reports invalid input.
    fn decode_utf8(bytes: &[u8], pos: usize) -> ParserResult<&str> {
        std::str::from_utf8(bytes).map_err(|error| ParserError::InvalidUtf8 {
            pos: pos + error.valid_up_to(),
        })
    }
}

/// Finds the end of the raw chunk starting at the front of `bytes`.
///
/// A raw chunk is a run of bytes that ends at the first byte needing special
/// handling: a C0 control character, a `"` (terminator) or a `\` (escape).
///
/// # Returns
/// `(chunk_len, is_ascii)`
/// - `chunk_len`: Length of the chunk, i.e. the pos of the first control char, `\` or `"`.
/// - `is_ascii` : Whether the chunk contains only ASCII chars.
#[inline]
fn find_raw_chunk_end(bytes: &[u8]) -> Option<(usize, bool)> {
    const MASK_ALL_NON_ASCII: u64 = 0x80_80_80_80_80_80_80_80;

    let mut byte_idx = 0;
    let mut all_ascii = true;
    let (words, remainder) = bytes.as_chunks::<8>();

    for word_bytes in words {
        let word = u64::from_le_bytes(*word_bytes);
        let mask_end_candidates = word.mask_raw_chunk_end();
        all_ascii &= word & MASK_ALL_NON_ASCII == 0;

        // Check if the word contains any bytes that could end the raw chunk
        if mask_end_candidates != 0 {
            let idx_first_candidate = mask_end_candidates.trailing_zeros() as usize / 8;
            return Some((byte_idx + idx_first_candidate, all_ascii));
        }

        byte_idx += u64::num_bytes();
    }

    for &byte in remainder {
        if byte >= 0x80 {
            all_ascii = false;
        } else if ends_raw_chunk(byte) {
            return Some((byte_idx, all_ascii));
        }
        byte_idx += 1;
    }

    None
}

trait WordExt {
    /// Marks the bytes of `self` that are candidates to end a raw chunk.
    ///
    /// Example:
    /// ```text
    /// chars: a   "   b   \   c   \n  d   e
    /// bytes: 61  22  62  5c  63  0a  64  65
    /// mask : 00  80  00  80  00  80  00  00   (0x80 = high bit set)
    /// ```
    fn mask_raw_chunk_end(self) -> Self;

    /// Marks the bytes of `self` that equal `needle` in their high bit.
    ///
    /// Example: (Only showing 5/8 bytes)
    /// ```text
    /// needle: "  (0x22)
    /// chars : a         "         b         "         c
    /// bytes : 0110_0001 0010_0010 0110_0010 0010_0010 0110_0011
    /// result: 0???_???? 1???_???? 0???_???? 1???_???? 0???_????   (only first bit relevant, '?' is noise)
    /// ```
    fn mask_byte(self, needle: u8) -> Self;

    fn num_bytes() -> usize;
}

impl WordExt for u64 {
    #[inline]
    fn mask_raw_chunk_end(self) -> Self {
        const MASK_HIGH_BITS: u64 = 0x80_80_80_80_80_80_80_80; // The high bit (0b1000_0000) of each byte
        const MIN_PRINTABLE: u64 = 0x20_20_20_20_20_20_20_20; // Smallest printable ASCII.

        // `A - 0x20` wraps to 0xe0..=0xff when A < 0x20 -> high bit = 1.
        // For A >= 0xa0 the high bit is set without wrapping (0xa0 - 0x20 = 0x80),
        // so `& !self` clears those high bits.
        //
        // byte - 0x20  & !byte -> high bit
        // 0x0a   0xea   0xe0      1 (end candidate)
        // 0x22   0x02   0x00      0 (ignore)
        // 0x5c   0x3c   0x20      0 (ignore)
        // 0x61   0x41   0x00      0 (ignore)
        let is_ctrl = self.wrapping_sub(MIN_PRINTABLE) & !self;

        // OR the three masks, then keep only the high bit of each lane, giving one
        // flag per byte:
        //
        // byte  is_ctrl '"'  '\'  mask
        // 0x0a  1        0    0   0x80  -> control char
        // 0x22  0        1    0   0x80  -> quote
        // 0x5c  0        0    1   0x80  -> backslash
        // 0x61  0        0    0   0x00  -> raw byte
        (is_ctrl | self.mask_byte(b'"') | self.mask_byte(b'\\')) & MASK_HIGH_BITS
    }

    #[inline]
    fn mask_byte(self, needle: u8) -> Self {
        const BROADCAST: u64 = 0x01_01_01_01_01_01_01_01;

        // needle = 0x22
        // byte ^ needle - 0x01 & !xor -> highest bit
        // 0x22   0x00     0xff   0xff    1 (matches needle)
        // 0x60   0x42     0x41   0x01    0 (does not match)
        let needles = u64::from(needle) * BROADCAST; // broadcast needle to each byte
        let xor = self ^ needles;
        xor.wrapping_sub(BROADCAST) & !xor
    }

    fn num_bytes() -> usize {
        std::mem::size_of::<u64>()
    }
}

/// Returns whether `byte` ends a raw chunk.
#[inline]
fn ends_raw_chunk(byte: u8) -> bool {
    byte.is_c0_control() || byte == b'"' || byte == b'\\'
}

#[cfg(test)]
mod tests {
    use crate::{
        from_bytes,
        parser::{
            Parser,
            error::{ParserErrorKind, ParserResult},
        },
        serde_de::Deserializer,
    };

    #[test]
    fn deserializes() {
        let borrowed = true;
        let owned = !borrowed;

        let cases_json: Vec<(&[u8], &str, bool, &str)> = vec![
            (br#""""#, "", borrowed, ""),
            (br#""hello""#, "hello", borrowed, ""),
        ];

        let cases_json_escapes: Vec<(&[u8], &str, bool, &str)> = vec![
            // --- Single char escape sequences ---
            (br#""[\"] [\\] [\/]""#, "[\"] [\\] [/]", owned, ""), // ", \ and /
            (br#""\n\r\t\b\f""#, "\n\r\t\u{0008}\u{000c}", owned, ""), // whitespace, backspace & form feed
            (
                // Mix of single char escapes + alpha
                br#""\ta\"]b\\c\/d\ne\rf\tg\bh\fi\n""#,
                "\ta\"]b\\c/d\ne\rf\tg\u{0008}h\u{000c}i\n",
                owned,
                "",
            ),
        ];

        let cases_djson: Vec<(&[u8], &str, bool, &str)> = vec![];

        let cases = cases_json
            .into_iter()
            .chain(cases_json_escapes)
            .chain(cases_djson);

        for (src, exp_value, exp_borrowed, exp_rest) in cases {
            let mut deserializer = Deserializer::new(src);

            if exp_borrowed {
                let actual_value = <&str as serde::Deserialize>::deserialize(&mut deserializer)
                    .expect("parses and can borrow");
                assert_eq!(actual_value, exp_value);

                from_bytes::<String>(src).expect("ok and converted to owned");
            } else {
                let actual_value = <String as serde::Deserialize>::deserialize(&mut deserializer)
                    .expect("parses and can borrow");
                assert_eq!(actual_value, exp_value);

                from_bytes::<&str>(src).expect_err("fails to parse or cannot borrow");
            }

            let src_utf8 = std::str::from_utf8(src).unwrap();
            let actual_rest = &src_utf8[deserializer.parser.pos..];
            assert_eq!(actual_rest, exp_rest);
        }
    }

    #[test]
    fn expect_errors() {
        type Case = (&'static str, fn(&str) -> ParserResult<()>, ParserErrorKind);

        let cases_json: &[Case] = &[
            // Missing opening quote
            (r#"abc""#, parse_str, ParserErrorKind::Expected),
            (r#" abc""#, parse_str, ParserErrorKind::Expected),
            // Missing closing quote
            (r#""abc"#, parse_str, ParserErrorKind::UnterminatedString),
            (r#""abc "#, parse_str, ParserErrorKind::UnterminatedString),
        ];

        let cases_json_escapes: &[Case] = &[
            (r#""\x""#, parse_str, ParserErrorKind::InvalidEscape), // Unknown
        ];

        let cases = cases_json.iter().chain(cases_json_escapes);

        for &(source, parse, expected) in cases {
            let actual = match parse(source) {
                Ok(()) => panic!("parsing `{source}` was meant to fail"),
                Err(error) => error.kind(),
            };
            assert_eq!(actual, expected);
        }
    }

    fn parse_str(src: &str) -> ParserResult<()> {
        Parser::new(src.as_bytes()).parse_str().map(drop)
    }
}
