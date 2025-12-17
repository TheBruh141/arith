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

    // Struct Errors
    UnknownType {
        span: Span,
        type_name: String,
    },
    UnknownField {
        span: Span,
        struct_name: String,
        field_name: String,
    },
    MissingField {
        span: Span,
        struct_name: String,
        field_name: String,
    },
    Generic {
        span: Span,
        message: String,
    },
    NotAStruct {
        span: Span,
        found: Type,
    },

    // General errors
    Custom {
        span: Span, // Make Custom errors span-aware too
        message: String,
    },
    UnknownVariant {
        span: Span,
        enum_name: String,
        variant_name: String,
    },
    ArityMismatch {
        span: Span,
        expected: usize,
        found: usize,
    },
    NotAnEnum {
        span: Span,
        found: Type,
    },
    PatternMismatch {
        span: Span,
        expected: Type,
        found: String, // Description of what we found (e.g., "Integer Literal")
    },
}

pub type CompileResult<T> = Result<T, CompileErr>;
