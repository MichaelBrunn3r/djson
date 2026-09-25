#[derive(Debug, thiserror::Error, miette::Diagnostic, PartialEq)]
pub enum ParserError {
    //region Numbers
    #[error("invalid integer")]
    InvalidInt {
        #[label("expected at least one digit")]
        pos: usize,
    },
    #[error("integer overflow for `{ty}`")]
    Overflow {
        #[label("does not fit in `{ty}`")]
        pos: usize,
        ty: &'static str,
    },
    //endregion Numbers
    //region Strings
    #[error("unterminated string")]
    UnterminatedString {
        #[label("missing closing `\"`")]
        pos: usize,
    },
    #[error("invalid string escape")]
    InvalidEscape {
        #[label("{msg}")]
        pos: usize,
        msg: &'static str,
    },
    #[error("invalid UTF-16 escape")]
    InvalidUtf16 {
        #[label("{msg}")]
        pos: usize,
        msg: &'static str,
    },
    #[error("unescaped control character")]
    ControlCharacter {
        #[label("`{byte:#04x}` must be escaped")]
        pos: usize,
        byte: u8,
    },
    #[error("invalid UTF-8")]
    InvalidUtf8 {
        #[label("not valid UTF-8")]
        pos: usize,
    },
    //endregion Strings
    #[error("missing separator")]
    MissingSeparator {
        #[label("missing separator")]
        pos: usize,
    },
    #[error("unknown identifier `{found}`")]
    InvalidIdentifier {
        #[label("not one of the expected identifiers")]
        pos: usize,
        found: String,
    },
    #[error("expected {expected}")]
    Expected {
        #[label("expected {expected}")]
        pos: usize,
        expected: &'static str,
    },
    #[cfg(feature = "serde")]
    #[error("{msg}")]
    Custom { msg: String },
}

#[cfg(feature = "serde")]
impl serde::de::Error for ParserError {
    fn custom<T: std::fmt::Display>(message: T) -> Self {
        Self::Custom {
            msg: message.to_string(),
        }
    }
}

pub type ParserResult<T> = Result<T, ParserError>;

//region Test utils
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParserErrorKind {
    // Numbers
    InvalidInt,
    Overflow,
    // Strings
    UnterminatedString,
    InvalidEscape,
    InvalidUtf16,
    ControlCharacter,
    InvalidUtf8,
    // Other
    Expected,
    InvalidIdentifier,
    MissingSeparator,
    Custom,
}

#[cfg(test)]
impl ParserError {
    pub(crate) fn kind(&self) -> ParserErrorKind {
        use crate::parser::error::ParserErrorKind::MissingSeparator;

        match self {
            // Numbers
            ParserError::InvalidInt { .. } => ParserErrorKind::InvalidInt,
            ParserError::Overflow { .. } => ParserErrorKind::Overflow,
            // Strings
            ParserError::InvalidEscape { .. } => ParserErrorKind::InvalidEscape,
            ParserError::UnterminatedString { .. } => ParserErrorKind::UnterminatedString,
            ParserError::InvalidUtf16 { .. } => ParserErrorKind::InvalidUtf16,
            ParserError::ControlCharacter { .. } => ParserErrorKind::ControlCharacter,
            ParserError::InvalidUtf8 { .. } => ParserErrorKind::InvalidUtf8,
            // Other
            ParserError::Expected { .. } => ParserErrorKind::Expected,
            ParserError::InvalidIdentifier { .. } => ParserErrorKind::InvalidIdentifier,
            ParserError::MissingSeparator { .. } => MissingSeparator,
            ParserError::Custom { .. } => ParserErrorKind::Custom,
        }
    }
}
//endregion Test utils
