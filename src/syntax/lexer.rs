use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone, Hash, Eq)]
#[logos(skip r"[ \t\n\f]+")] // Skip whitespace
#[logos(skip r"//[^\n]*")] // Skip comments starting with // until newline
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
    #[token("assert")]
    Assert,

    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("match")]
    Match,

    // Type Keywords
    #[token("Int")]
    TInt, // BigInt
    #[token("i8")]
    Ti8,
    #[token("i16")]
    Ti16,
    #[token("i32")]
    Ti32,
    #[token("i64")]
    Ti64,
    #[token("isize")]
    Tisize,

    #[token("u8")]
    Tu8,
    #[token("u16")]
    Tu16,
    #[token("u32")]
    Tu32,
    #[token("u64")]
    Tu64,
    #[token("usize")]
    Tusize,

    #[token("f16")]
    Tf16,
    #[token("f32")]
    Tf32,
    #[token("f64")]
    Tf64,

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
    #[token(".")]
    Dot,
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

    #[token(";")]
    Semi,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // Comparators
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("!")]
    Bang,
    #[token(">=")]
    Geq,
    #[token("<=")]
    Leq,

    #[token("...")]
    TripleDot,

    // Literals
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Numeric Literals with Suffix Support
    // We parse everything as a string first to handle the suffixes in parser/AST
    // or we can distinct via regex here.
    // Let's rely on regex complexity to capture typed literals.

    // 123u8
    #[regex("[0-9]+u8", |lex| lex.slice().trim_end_matches("u8").parse().ok())]
    LitU8(u8),
    #[regex("[0-9]+u16", |lex| lex.slice().trim_end_matches("u16").parse().ok())]
    LitU16(u16),
    #[regex("[0-9]+u32", |lex| lex.slice().trim_end_matches("u32").parse().ok())]
    LitU32(u32),
    #[regex("[0-9]+u64", |lex| lex.slice().trim_end_matches("u64").parse().ok())]
    LitU64(u64),
    #[regex("[0-9]+usize", |lex| lex.slice().trim_end_matches("usize").parse().ok())]
    LitUsize(usize),

    #[regex("[0-9]+i8", |lex| lex.slice().trim_end_matches("i8").parse().ok())]
    LitI8(i8),
    #[regex("[0-9]+i16", |lex| lex.slice().trim_end_matches("i16").parse().ok())]
    LitI16(i16),
    #[regex("[0-9]+i32", |lex| lex.slice().trim_end_matches("i32").parse().ok())]
    LitI32(i32),
    #[regex("[0-9]+i64", |lex| lex.slice().trim_end_matches("i64").parse().ok())]
    LitI64(i64),
    #[regex("[0-9]+isize", |lex| lex.slice().trim_end_matches("isize").parse().ok())]
    LitIsize(isize),

    // Floating Points
    #[regex("[0-9]+\\.[0-9]+f64", |lex| lex.slice().trim_end_matches("f64").to_string())]
    #[regex("[0-9]+\\.[0-9]+", |lex| lex.slice().to_string())] // Default float is f64
    LitF64(String),

    #[regex("[0-9]+\\.[0-9]+f32", |lex| lex.slice().trim_end_matches("f32").to_string())]
    LitF32(String),

    #[regex("[0-9]+\\.[0-9]+f16", |lex| lex.slice().trim_end_matches("f16").to_string())]
    LitF16(String),

    // Plain Integer (BigInt/Int default)
    // We store as String to parse into BigInt later, OR use i64 if it fits?
    // User wants BigInt default.
    // Let's use a String for now to be safe and parse in AST conversion.
    #[regex("[0-9]+", |lex| lex.slice().to_string())]
    LitInt(String),

    // Comments
    #[regex(r"--\[\[(?:[^\]]|\][^\]])*\]\]", logos::skip)]
    BlockComment,
    #[regex(r"--[^\n]*", logos::skip, allow_greedy = true)]
    LineComment,

    Underscore,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
