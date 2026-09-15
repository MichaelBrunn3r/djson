#[cfg(any(test, feature = "pretty_ast"))]
pub mod pretty;

use crate::span::Spanned;

#[derive(Debug, PartialEq)]
pub struct AST<'input> {
    pub statements: Vec<Spanned<Statement<'input>>>,
}

#[derive(Debug, PartialEq)]
pub enum Statement<'input> {
    Expr(Spanned<Expr<'input>>),
    KV(Spanned<KV<'input>>),
    Let(Spanned<Let<'input>>),
}

#[derive(Debug, PartialEq)]
pub struct Let<'input> {
    pub pattern: Spanned<Pattern<'input>>,
    pub expr: Spanned<Expr<'input>>,
}

#[derive(Debug, PartialEq)]
pub enum Pattern<'input> {
    Name(&'input str),
    Map(Vec<Spanned<MapPattern<'input>>>),
    List {
        patterns: Vec<Spanned<Self>>,
        rest: Option<Spanned<&'input str>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct MapPattern<'input> {
    pub key: &'input str,
    pub pattern: Spanned<Pattern<'input>>,
    pub default: Option<Spanned<Expr<'input>>>,
}

#[derive(Debug, PartialEq)]
pub struct KV<'input> {
    pub key: Spanned<&'input str>,
    pub expr: Spanned<Expr<'input>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Identifier<'input> {
    Simple(&'input str),
}

#[derive(Debug, PartialEq)]
pub enum Expr<'input> {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(&'input str),
    List(Vec<Spanned<Self>>),
    Map(Vec<Spanned<KV<'input>>>),
    Id(Identifier<'input>),
    Access {
        object: Box<Spanned<Self>>,
        name: &'input str,
    },
    Unary {
        op: PrefixOp,
        value: Box<Spanned<Self>>,
    },
    Binary {
        left: Box<Spanned<Self>>,
        op: InfixOp,
        right: Box<Spanned<Self>>,
    },
    Call {
        callee: Box<Spanned<Self>>,
        arguments: Vec<Spanned<Self>>,
    },
    If {
        condition: Box<Spanned<Self>>,
        then: Box<Spanned<Self>>,
        r#else: Box<Spanned<Self>>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrefixOp {
    Positive,
    Negative,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Equal,
}
