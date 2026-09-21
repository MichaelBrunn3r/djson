mod parser;
#[cfg(feature = "serde")]
mod serde_de;
#[cfg(test)]
mod test_utils;
mod utils;

pub use parser::{ParserError, ParserResult};
#[cfg(feature = "serde")]
pub use serde_de::from_bytes;
