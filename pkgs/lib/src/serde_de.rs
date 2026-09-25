use serde::de::value::BorrowedStrDeserializer;

use crate::{
    parser::error::ParserError,
    parser::{ParsedStr, Parser},
};

pub struct Deserializer<'de> {
    pub(crate) parser: Parser<'de>,
}

impl<'de> Deserializer<'de> {
    /// Creates a deserializer for `src`.
    #[must_use]
    pub fn new(src: &'de [u8]) -> Self {
        Self {
            parser: Parser::new(src),
        }
    }
}

macro_rules! impl_deserialize_uint {
    ($($method:ident => $ty:ty, $visit:ident;)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                let value: $ty = self.parser.parse_uint()?;
                visitor.$visit(value)
            }
        )*
    };
}

impl<'de> serde::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = ParserError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unimplemented!()
    }

    impl_deserialize_uint! {
        deserialize_u8 => u8, visit_u8;
        deserialize_u16 => u16, visit_u16;
        deserialize_u32 => u32, visit_u32;
        deserialize_u64 => u64, visit_u64;
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.parser.parse_str()? {
            ParsedStr::Borrowed(str) => visitor.visit_borrowed_str(str),
            ParsedStr::Scratch(str) => visitor.visit_str(str),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.parser.expect_list_start()?;
        visitor.visit_seq(ListAccess {
            de: self,
            first: true,
        })
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.parser.expect_map_start()?;
        visitor.visit_map(StructAccess::new(self, fields))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 f32 f64
        char bytes byte_buf option unit unit_struct
        map newtype_struct enum identifier ignored_any
    }
}

/// Visits the items of a list whose opening `[` has already been consumed.
struct ListAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'de> serde::de::SeqAccess<'de> for ListAccess<'_, 'de> {
    type Error = ParserError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if !self.de.parser.advance_to_next_item(self.first)? {
            return Ok(None);
        }
        self.first = false;

        seed.deserialize(&mut *self.de).map(Some)
    }
}

//region StructAccess
struct StructAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    fields: &'static [&'static str],
    first: bool,
}

impl<'a, 'de> StructAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, fields: &'static [&'static str]) -> Self {
        Self {
            de,
            fields,
            first: true,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for StructAccess<'_, 'de> {
    type Error = ParserError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if !self.de.parser.advance_to_next_kv(self.first)? {
            return Ok(None);
        }
        self.first = false;

        let field = self.de.parser.parse_known_key(self.fields)?;

        self.de.parser.expect_kv_separator()?;
        seed.deserialize(BorrowedStrDeserializer::new(field))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}
//endregion StructAccess

#[cfg(test)]
mod tests {
    use bumpalo::Bump;
    use serde::de::DeserializeSeed;

    use super::*;
    use crate::{
        ParserResult, from_bytes,
        utils::test::{ArenaString, discard_ok, fmt_report, fmt_snapshot_case, fmt_snapshot_cases},
    };

    #[test]
    fn diagnostics() {
        let cases: &[(&str, &str, fn(&[u8]) -> ParserResult<()>)] = &[
            ("not a list", "1", discard_ok!(from_bytes::<Vec<u8>>)),
            (
                "missing separator",
                "[1 2]",
                discard_ok!(from_bytes::<Vec<u8>>),
            ),
            ("unclosed list", "[1, 2", discard_ok!(from_bytes::<Vec<u8>>)),
        ];

        let snap = fmt_snapshot_cases(cases, |&(label, source_code, parse)| {
            let error = parse(source_code.as_bytes())
                .err()
                .expect("case is meant to fail");
            let report = miette::Report::from(error).with_source_code(source_code);
            fmt_snapshot_case(label, &[("error", &fmt_report(&report))])
        });
        insta::assert_snapshot!(snap);
    }

    #[test]
    fn deserializes_with_seed() {
        let arena = Bump::new();
        assert_eq!(arena.allocated_bytes(), 0);

        let mut deserializer = Deserializer::new(br#""hello\nworld""#);
        let value: &str = ArenaString { arena: &arena }
            .deserialize(&mut deserializer)
            .expect("seed deserializes");

        assert_eq!(value, "hello\nworld");
        assert!(arena.allocated_bytes() > 0);
    }
}
