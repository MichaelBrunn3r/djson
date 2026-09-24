use crate::parser::{
    Parser,
    error::{ParserError, ParserResult},
};

impl<'src> Parser<'src> {
    /// Decodes a UTF-16 code point escape to a utf8 char.
    ///
    /// # Grammar
    /// ```text
    /// utf16Escape     = \uXXXX | highSurrogate lowSurrogate
    /// highSurrogate   = \uXXXX
    /// lowSurrogate    = \uXXXX
    /// ```
    pub(super) fn parse_escaped_utf16_char(&mut self, start: usize) -> ParserResult<char> {
        debug_assert_eq!(
            self.src.get(self.pos - 2..self.pos),
            Some(b"\\u".as_slice())
        );

        const HIGH_SUR_MIN: u16 = 0xD800; // 0xD800..=0xDBFF (reserved high surrogates code points)
        const LOW_SUR_MIN: u16 = 0xDC00; // 0xDC00..=0xDFFF (reserved low surrogates code points)
        const SURROGATE_SIZE: u16 = (0xDBFF - 0xD800) + 1; // = 0x400

        let unit = self.parse_utf16_code_unit()?;

        // Map surrogates to the bottom of the u16 range:
        // - char       : 0x0000..=0xD7FF -> 0x2800..=0xFFFF >= 2*0x400 = 0x0800
        // - high sur.  : 0xD800..=0xDBFF -> 0x0000..0x0400  < 0x0800
        // - low sur.   : 0xDC00..=0xDFFF -> 0x0400..0x0800  < 0x0800
        // - char       : 0xE000..=0xFFFF -> 0x0800..0x2800  >= 0x0800
        let high = unit.wrapping_sub(HIGH_SUR_MIN);

        // Check if `unit` is a valid char
        if high >= 2 * SURROGATE_SIZE {
            // SAFETY: `unit` is not a surrogate, and `0xFFFF` (=u16::MAX) is below `0x10FFFF`
            // (largest code point), so `unit` is a valid `char`.
            return Ok(unsafe { char::from_u32_unchecked(u32::from(unit)) });
        }

        // `unit` must be a high surrogate followed by the `\u` of the low surrogate
        if !(high < SURROGATE_SIZE && self.expect_next_bytes(b"\\u")) {
            return Err(ParserError::InvalidUtf16 {
                pos: start,
                msg: "a low surrogate must follow a high surrogate",
            });
        }

        let low = self.parse_utf16_code_unit()?.wrapping_sub(LOW_SUR_MIN);
        if low >= SURROGATE_SIZE {
            return Err(ParserError::InvalidUtf16 {
                pos: start,
                msg: "a low surrogate must follow a high surrogate",
            });
        }

        let value = 0x1_0000 + (u32::from(high) << 10) + u32::from(low);
        // SAFETY: 0 <= (low & high) < 0x400, so `value` lies in `0x10000..=0x10FFFF` -> valid `char`.
        Ok(unsafe { char::from_u32_unchecked(value) })
    }

    /// Parse the 4 hex digits of a `\uXXXX` whose `\u` to a u16.
    fn parse_utf16_code_unit(&mut self) -> ParserResult<u16> {
        debug_assert_eq!(
            self.src.get(self.pos - 2..self.pos),
            Some(b"\\u".as_slice())
        );
        let start = self.pos;

        let digits: &[u8; 4] = self
            .src
            .get(start..start + 4)
            .and_then(|digits| digits.try_into().ok())
            .ok_or(ParserError::InvalidUtf16 {
                pos: self.src.len(),
                msg: "expected 4 hex digits",
            })?;

        let nibbles = digits.map(|byte| HEX_LUT[usize::from(byte)]);

        // Pack the 4 nibbles into 2 bytes -> fits into a u16.
        // Non hex digits poison the result so it no longer fits in a u16.
        let unit = (nibbles[0] << 12) | (nibbles[1] << 8) | (nibbles[2] << 4) | nibbles[3];
        let unit = u16::try_from(unit).ok().ok_or(ParserError::InvalidUtf16 {
            pos: start,
            msg: "expected 4 hex digits",
        })?;
        self.pos = start + 4;

        Ok(unit)
    }
}

/// Maps every byte in `0-9a-fA-F` to its nibble value. Every other byte maps to `u32::MAX`.
const HEX_LUT: [u32; 256] = {
    let mut lut = [u32::MAX; 256];

    let mut digit = 0u8;
    while digit < 10 {
        lut[(b'0' + digit) as usize] = digit as u32;
        digit += 1;
    }

    let mut letter = 0u8;
    while letter < 6 {
        let digit = (10 + letter) as u32;
        lut[(b'a' + letter) as usize] = digit;
        lut[(b'A' + letter) as usize] = digit;
        letter += 1;
    }

    lut
};

#[cfg(test)]
mod tests {
    use crate::{
        parser::{
            Parser,
            error::{ParserErrorKind, ParserResult},
        },
        serde_de::{Deserializer, from_bytes},
    };

    #[test]
    fn deserializes() {
        let borrowed = true;
        let owned = !borrowed;

        let cases: Vec<(&[u8], &str, bool, &str)> = vec![
            // --- UTF-16 ---
            (br#""\u30ad\u30AD\u0041""#, "キキA", owned, ""), // case insensitive
            (br#""\ud83d\ude00\uD83D\uDE00""#, "😀😀", owned, ""), // only valid as a pair
            (
                //   0000-FFFF,    1_0000-10_ffff
                br#""\u0000-\uffff \ud800\udc00-\udbff\udfff""#,
                "\u{0000}-\u{ffff} \u{10000}-\u{10ffff}",
                owned,
                "",
            ),
        ];

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

        let cases_json_utf16: &[Case] = &[
            // `\u` not followed by four hexadecimal digits
            (r#""\u""#, parse_str, ParserErrorKind::InvalidUtf16),
            (r#""\u12""#, parse_str, ParserErrorKind::InvalidUtf16),
            (r#""\u12g4""#, parse_str, ParserErrorKind::InvalidUtf16), // g is not hex
        ];

        let cases_djson_utf16: &[Case] = &[
            // Reject lone surrogate pairs
            (r#""\ud800""#, parse_str, ParserErrorKind::InvalidUtf16),
            (r#""\udbff""#, parse_str, ParserErrorKind::InvalidUtf16),
            (r#""\udc00""#, parse_str, ParserErrorKind::InvalidUtf16),
            (r#""\udfff""#, parse_str, ParserErrorKind::InvalidUtf16),
            // Reject pairs in wrong surrogate order
            (
                r#""\ud800\u0041""#,
                parse_str,
                ParserErrorKind::InvalidUtf16,
            ),
            (
                r#""\ud800\ud800""#,
                parse_str,
                ParserErrorKind::InvalidUtf16,
            ),
            (r#""\ud800x""#, parse_str, ParserErrorKind::InvalidUtf16),
        ];

        let cases = cases_json_utf16.iter().chain(cases_djson_utf16);

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
