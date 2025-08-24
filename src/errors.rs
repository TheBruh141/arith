use crate::tokenizer::TokenType;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnexpectedToken(TokenType),
    UnexpectedEOF,
    InvalidNumber(String),
    TokenizerError(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken(found) => {
                write!(f, "Unexpected token: found {}", found)
            }
            ParserError::UnexpectedEOF => write!(f, "Unexpected end of input"),
            ParserError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ParserError::TokenizerError(s) => write!(f, "Tokenizer error: {}", s),
        }
    }
}