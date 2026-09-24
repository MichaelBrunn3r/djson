//! Helpers shared by the benchmark binaries.

use std::{fs, path::PathBuf};

use bumpalo::Bump;
use serde::de::{DeserializeSeed, Deserializer, Visitor};

/// Reads the document named `name` from the bench resource directory.
pub fn read_bench_resource(name: &str) -> Vec<u8> {
    let path = resolve_bench_resource(name);
    fs::read(&path).unwrap_or_else(|error| panic!("reading `{}`: {error}", path.display()))
}

/// Absolute path of `name` in the bench resource directory.
pub fn resolve_bench_resource(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches/res")
        .join(name)
}

#[derive(Clone, Copy)]
pub struct ArenaString<'arena> {
    pub arena: &'arena Bump,
}

impl<'de, 'arena> DeserializeSeed<'de> for ArenaString<'arena> {
    type Value = &'arena str;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
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
