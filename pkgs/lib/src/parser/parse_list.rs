use crate::{
    ParserResult,
    parser::{Parser, error::ParserError},
};

impl<'src> Parser<'src> {
    /// Consumes the start of a list.
    ///
    /// ## Errors
    /// The following bytes do not start a list
    pub(crate) fn expect_list_start(&mut self) -> ParserResult<()> {
        self.skip_whitespace();
        self.expect(b'[', "`[`")
    }

    /// Consumes the end of a list.
    #[inline]
    fn eat_list_end(&mut self) -> bool {
        self.eat(b']')
    }

    /// Advances to the next item of the list
    pub(crate) fn advance_to_next_item(&mut self, first: bool) -> ParserResult<bool> {
        self.skip_whitespace();
        if !first {
            if self.eat_list_end() {
                return Ok(false);
            }
            self.expect(b',', "`,` or `]`")?;
            self.skip_whitespace();
        }

        if self.eat_list_end() {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::assert_deserializes_cases;

    #[test]
    fn deserializes_lists() {
        assert_deserializes_cases! {
            "[1, 2, 3]" => vec![1u32, 2, 3],
            "[[1, 2], [], [3]]" => vec![vec![1u8, 2], vec![], vec![3]], // Nested
            "[\n \t 1  , \n    \r2,\n]" => vec![1u8, 2], // Whitespace
            r#"["a", "b\", c"]"# => vec![String::from("a"), String::from("b\", c")],
        }
    }
}
