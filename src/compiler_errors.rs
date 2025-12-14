use crate::syntax::ast::{Span, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum CompileErr {
    UnexpectedToken {
        span: Span,
        expected: Vec<Option<String>>,
        found: Option<String>,
    },
    UnclosedDelimiter {
        span: Span,
        delimiter: String,
    },
    TypeMismatch {
        span: Span,
        expected: Type,
        found: Type,
    },
    UnknownVar {
        span: Span,
        name: String,
    },
    Custom(String),
}

pub type CompileResult<T> = Result<T, CompileErr>;
