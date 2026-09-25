#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ParserError {
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
    #[error("expected {expected}")]
    Expected {
        #[label("expected {expected}")]
        pos: usize,
        expected: &'static str,
    },
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
    #[error("unknown identifier `{found}`")]
    InvalidIdentifier {
        #[label("not one of the expected identifiers")]
        pos: usize,
        found: String,
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
    InvalidInt,
    Overflow,
    Expected,
    UnterminatedString,
    InvalidEscape,
    InvalidUtf16,
    ControlCharacter,
    InvalidUtf8,
    InvalidIdentifier,
    Custom,
}

#[cfg(test)]
impl ParserError {
    pub(crate) fn kind(&self) -> ParserErrorKind {
        match self {
            ParserError::InvalidInt { .. } => ParserErrorKind::InvalidInt,
            ParserError::Overflow { .. } => ParserErrorKind::Overflow,
            ParserError::Expected { .. } => ParserErrorKind::Expected,
            ParserError::UnterminatedString { .. } => ParserErrorKind::UnterminatedString,
            ParserError::InvalidEscape { .. } => ParserErrorKind::InvalidEscape,
            ParserError::InvalidUtf16 { .. } => ParserErrorKind::InvalidUtf16,
            ParserError::ControlCharacter { .. } => ParserErrorKind::ControlCharacter,
            ParserError::InvalidUtf8 { .. } => ParserErrorKind::InvalidUtf8,
            ParserError::InvalidIdentifier { .. } => ParserErrorKind::InvalidIdentifier,
            ParserError::Custom { .. } => ParserErrorKind::Custom,
        }
    }
}
//endregion Test utils
