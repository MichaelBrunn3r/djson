mod parser;
mod utils;

#[cfg(feature = "serde")]
mod serde_de;

pub use crate::{parser::error::ParserResult, serde_de::Deserializer};

/// Deserializes `src` as a single djson document.
///
/// # Errors
///
/// Returns a [`ParserError`] if `src` is not a well-formed document or does not
/// describe a `T`.
#[must_use]
pub fn from_bytes<'src, T>(src: &'src [u8]) -> ParserResult<T>
where
    T: serde::Deserialize<'src>,
{
    let mut deserializer = Deserializer::new(src);
    let value = T::deserialize(&mut deserializer)?;
    Ok(value)
}
