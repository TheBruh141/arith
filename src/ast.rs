//! This module defines the Abstract Syntax Tree (AST) nodes for the `arith` language.
//!
//! While this file contains the `ASTNode` enum, the primary AST representation
//! currently used throughout the project (in `parser.rs` and `executor.rs`)
//! is the `Expr` enum defined in `src/parser.rs`.
//!
//! This `ASTNode` enum might be a placeholder for future extensions or
//! an alternative AST structure.

pub enum ASTNode {
    /// Represents a numeric literal in the AST.
    Number(f64),
    /// Represents a variable. (Currently unused in the main interpreter flow).
    Variable(String),
    /// Represents an addition operation. (Currently unused in the main interpreter flow).
    Addition {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    /// Represents a subtraction operation. (Currently unused in the main interpreter flow).
    Subtraction {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    /// Represents a multiplication operation. (Currently unused in the main interpreter flow).
    Multiplication {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    /// Represents a division operation. (Currently unused in the main interpreter flow).
    Division {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    /// Represents a unary minus operation. (Currently unused in the main interpreter flow).
    UnaryMinus(Box<ASTNode>),
    // FunctionCall { name: String, args: Vec<Node> }, // Example of a potential future extension
}
