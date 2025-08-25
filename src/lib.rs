//! `arith` is a lightweight and efficient command-line interpreter for arithmetic expressions.
//!
//! It provides a simple yet powerful way to evaluate mathematical expressions directly from the terminal,
//! supporting basic operations, operator precedence, implicit multiplication, and an interactive
//! Read-Eval-Print Loop (REPL).
//!
//! The interpreter pipeline consists of:
//! - **Tokenization:** Breaking down the input string into a sequence of tokens (`tokenizer` module).
//! - **Parsing:** Constructing an Abstract Syntax Tree (AST) from the tokens (`parser` module).
//! - **Execution:** Evaluating the AST to produce a result (`executor` module).
//!
//! This crate also provides modules for error handling (`errors`), AST definition (`ast`),
//! REPL functionality (`repl`), and file-based execution (`filemode`).

pub mod ast;
pub mod errors;
pub mod executor;
pub mod parser;
pub mod repl;
pub mod tokenizer;

pub mod filemode; // Declare the new module
