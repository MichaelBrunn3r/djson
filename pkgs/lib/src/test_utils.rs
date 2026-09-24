use bumpalo::Bump;
use serde::de::{DeserializeSeed, Deserializer as SerdeDeserializer, Visitor};

use crate::{from_bytes, parser::Parser};

pub(crate) fn fmt_report(report: &miette::Report) -> String {
    let handler =
        miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
    let mut rendered = String::new();
    handler
        .render_report(&mut rendered, report.as_ref())
        .expect("writing into a `String` cannot fail");
    rendered
}

#[must_use]
pub fn fmt_snapshot_cases<Cases, Case, Format>(cases: Cases, format: Format) -> String
where
    Cases: IntoIterator<Item = Case>,
    Format: FnMut(Case) -> String,
{
    cases
        .into_iter()
        .map(format)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn fmt_snapshot_case(label: &str, fields: &[(&str, &str)]) -> String {
    let fields = fields
        .iter()
        .map(|(label, value)| {
            let value = value.trim_end().replace('\n', "\n        ");
            format!("{label}: `{value}`")
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{label}\n{fields}")
}

/// Turns `(arg) -> Result<T, E>` into `(arg) -> Result<(), E>`.
macro_rules! discard_ok {
    ($fn:path) => {
        |arg| $fn(arg).map(|_| ())
    };
}
pub(crate) use discard_ok;

//region assert_deserializes
pub(crate) fn assert_deserializes<T>(src: &str, expected: T)
where
    T: serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let actual = from_bytes::<T>(src.as_bytes()).expect("parses");
    assert_eq!(actual, expected, "parsing `{src}`");
}

/// Asserts that each case parses into its expected value.
macro_rules! assert_deserializes_cases {
    ($($source:literal => $expected:expr),* $(,)?) => {
        $(
            crate::test_utils::assert_deserializes($source, $expected);
        )*
    };
}
pub(crate) use assert_deserializes_cases;
//endregion assert_deserializes

//region ParserExt
pub(crate) trait ParserExt {
    fn assert_remaining_src(&self, expected: &str);
}

impl<'src> ParserExt for Parser<'src> {
    fn assert_remaining_src(&self, expected: &str) {
        let src = std::str::from_utf8(&self.src).expect("src is utf8");
        let rest = &src[self.pos..];
        assert_eq!(rest, expected, "`{src}` was consumed entirely");
    }
}
//endregion ParserExt

//region Deserialize with seed
pub(crate) struct ArenaString<'arena> {
    pub(crate) arena: &'arena Bump,
}

impl<'de, 'arena> DeserializeSeed<'de> for ArenaString<'arena> {
    type Value = &'arena str;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: SerdeDeserializer<'de>,
    {
        deserializer.deserialize_str(ArenaStringVisitor { arena: self.arena })
    }
}

struct ArenaStringVisitor<'arena> {
    arena: &'arena Bump,
}

impl<'de, 'arena> Visitor<'de> for ArenaStringVisitor<'arena> {
    type Value = &'arena str;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.arena.alloc_str(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.arena.alloc_str(value))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.arena.alloc_str(&value))
    }
}
//endregion Deserialize with seed
