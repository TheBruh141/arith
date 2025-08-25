use crate::tokenizer::TokenType;
use std::fmt;

/// Represents an error that can occur during parsing.
#[derive(Debug, PartialEq)]
pub enum ParserError {
    /// An unexpected token was found.
    UnexpectedToken {
        found: TokenType,
        line: usize,
        col: usize,
    },
    /// The input ended unexpectedly.
    UnexpectedEOF {
        line: usize,
        col: usize,
    },
    /// An invalid number format was found.
    InvalidNumber {
        value: String,
        line: usize,
        col: usize,
    },
    /// An error occurred during tokenization.
    TokenizerError {
        message: String,
        line: usize,
        col: usize,
    },
}

/// Represents an error that can occur during tokenization.
#[derive(Debug, PartialEq)]
pub enum TokenizerError {
    /// An unexpected character was found.
    UnexpectedCharacter {
        found: char,
        line: usize,
        col: usize,
    },
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerError::UnexpectedCharacter { found, line, col } => {
                write!(f, "Unexpected character '{}' at line {}, col {}", found, line, col)
            }
        }
    }
}

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
