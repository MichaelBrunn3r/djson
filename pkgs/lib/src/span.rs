use miette::SourceSpan;

/// A value paired with the span of the source text it was parsed from.
#[derive(Debug, PartialEq, Eq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: SourceSpan,
}
