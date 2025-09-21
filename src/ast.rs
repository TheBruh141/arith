//! This module defines the Abstract Syntax Tree (AST) for the `arith` language.
//!
//! The `Statement` and `Expr` enums represent the grammatical structure of the code.

use crate::tokenizer::TokenType;

/// Represents a statement in the `arith` language.
#[derive(Debug, PartialEq)]
pub enum Statement {
    /// An expression statement, e.g., `1 + 2`.
    Expression(Expr),
    /// A `let` statement, e.g., `let x: Int = 10`.
    Let {
        name: String,
        type_name: Option<String>,
        value: Expr,
    },
    /// An assignment statement, e.g., `x = 10`.
    Assignment { name: String, value: Expr },
}

/// Represents an expression in the `arith` language.
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// A literal floating-point number, e.g., `42.0`, `3.14`.
    Number(f64),

    /// An identifier, representing a variable reference, e.g., `x`.
    Variable(String),

    /// A unary operation, e.g., `-5`, `+x`.
    UnaryOp { op: TokenType, expr: Box<Expr> },

    /// A binary operation, e.g., `a + b`, `c * d`.
    BinaryOp {
        left: Box<Expr>,
        op: TokenType,
        right: Box<Expr>,
    },

    /// Represents an empty expression, typically from an empty input string.
    Empty,

    /// Represents empty parentheses, e.g., `()`. In `arith`, this evaluates to `0`.
    EmptyParen,
}
