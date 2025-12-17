use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone, Hash, Eq)]
#[logos(skip r"[ \t\n\f]+")] // Skip whitespace
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("rec")]
    Rec,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Type Keywords
    #[token("Int")]
    TInt,
    #[token("Bool")]
    TBool,
    #[token("Vector")]
    TVector,

    // Symbols
    #[token(r".\")]
    Lambda, // .\
    #[token("->")]
    Arrow,
    #[token(":")]
    Colon,
    #[token("=")]
    Eq, // Assignment
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token(",")]
    Comma,

    // Comparators
    #[token("==")]
    EqEq,
    #[token(">=")]
    Geq,
    #[token("<=")]
    Leq,

    // Literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    Num(i64),

    // Comments
    #[regex(r"--\[\[(?:[^\]]|\][^\]])*\]\]", logos::skip)]
    BlockComment,
    #[regex(r"--[^\n]*", logos::skip, allow_greedy = true)]
    LineComment,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
