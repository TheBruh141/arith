//! This module defines the custom error types used throughout the `arith` interpreter.
//!
//! These errors provide detailed information, including the location (line and column)
//! where the error occurred, to facilitate debugging and user feedback.

use crate::tokenizer::TokenType;
use std::fmt;

/// Represents an error that can occur during parsing.
///
/// This enum covers various syntax-related issues detected by the parser,
/// such as unexpected tokens, premature end of input, or malformed numbers.
#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An unexpected token was encountered during parsing.
    ///
    /// `found`: The `TokenType` that was found but not expected at the current position.
    /// `line`: The 1-based line number where the unexpected token was found.
    /// `col`: The 1-based column number where the unexpected token starts.
    UnexpectedToken {
        found: TokenType,
        line: usize,
        col: usize,
    },
    /// The end of the input was reached prematurely, indicating an incomplete expression.
    ///
    /// `line`: The 1-based line number where the unexpected EOF occurred.
    /// `col`: The 1-based column number where the unexpected EOF occurred.
    UnexpectedEOF {
        line: usize,
        col: usize,
    },
    /// A number literal was found but could not be parsed into a valid floating-point number.
    ///
    /// `value`: The string representation of the invalid number.
    /// `line`: The 1-based line number where the invalid number was found.
    /// `col`: The 1-based column number where the invalid number starts.
    InvalidNumber {
        value: String,
        line: usize,
        col: usize,
    },
    /// An error propagated from the tokenizer during the parsing process.
    ///
    /// This variant wraps a message from the `TokenizerError` and its location.
    /// `message`: A descriptive message from the tokenizer error.
    /// `line`: The 1-based line number where the tokenizer error occurred.
    /// `col`: The 1-based column number where the tokenizer error occurred.
    TokenizerError {
        message: String,
        line: usize,
        col: usize,
    },
}

/// Represents an error that can occur during the tokenization (lexical analysis) phase.
///
/// This enum primarily handles cases where an unrecognized character is encountered
/// in the input string.
#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    /// An unexpected or unrecognized character was found in the input string.
    ///
    /// `found`: The character that caused the error.
    /// `line`: The 1-based line number where the unexpected character was found.
    /// `col`: The 1-based column number where the unexpected character was found.
    UnexpectedCharacter {
        found: char,
        line: usize,
        col: usize,
    },
}

/// Implements the `Display` trait for `TokenizerError`, allowing it to be
/// easily formatted as a user-friendly string.
impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerError::UnexpectedCharacter { found, line, col } => {
                write!(f, "Unexpected character '{}' at line {}, col {}", found, line, col)
            }
        }
    }
}

/// Implements the `Display` trait for `ParserError`, allowing it to be
/// easily formatted as a user-friendly string.
impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken { found, line, col } => {
                write!(f, "Unexpected token: found {} at line {}, col {}", found, line, col)
            }
            ParserError::UnexpectedEOF { line, col } => {
                write!(f, "Unexpected end of input at line {}, col {}", line, col)
            }
            ParserError::InvalidNumber { value, line, col } => {
                write!(f, "Invalid number: {} at line {}, col {}", value, line, col)
            }
            ParserError::TokenizerError { message, line, col } => {
                write!(f, "Tokenizer error: {} at line {}, col {}", message, line, col)
            }
        }
    }
}
