use crate::syntax::ast::{BinaryOp, Span, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum CompileErr {
    // Parser errors
    UnexpectedToken {
        span: Span,
        expected: Vec<Option<String>>,
        found: Option<String>,
    },
    UnclosedDelimiter {
        span: Span,
        delimiter: String,
    },

    // Type checker errors
    TypeMismatch {
        // Generic type mismatch, still useful for cases not covered by specific errors
        span: Span,
        expected: Type,
        found: Type,
    },
    NotAFunction {
        // Specific to application of non-functions
        span: Span,
        found_type: Type,
    },
    BinaryOpMismatch {
        // Specific to binary operations operands
        span: Span,
        op: BinaryOp,
        expected_operand_type: Type,
        found_operand_type: Type,
    },
    IfConditionNotBool {
        // Specific to 'if' condition
        span: Span,
        found_type: Type,
    },
    IfBranchesMismatch {
        // Specific to 'if' branches
        span: Span,
        then_type: Type,
        else_type: Type,
    },

    // Scope errors
    UnknownVar {
        span: Span,
        name: String,
    },

    // General errors
    Custom {
        span: Span, // Make Custom errors span-aware too
        message: String,
    },
}

pub type CompileResult<T> = Result<T, CompileErr>;
