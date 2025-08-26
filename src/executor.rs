//! This module is responsible for the compilation and execution of arithmetic expressions.
//!
//! It defines the bytecode instructions, a compiler to transform the Abstract Syntax Tree (AST)
//! into bytecode, and a simple stack-based executor to evaluate the bytecode.
//! It also orchestrates the entire evaluation pipeline, from raw input string to final result,
//! handling line continuations, comments, and comprehensive error reporting.

use crate::errors::{ParserError, TokenizerError};
use crate::parser::{Expr, Parser};
use crate::tokenizer::{TokenType, Tokenizer};
use std::error::Error;
use std::fmt;

/// Represents a single bytecode instruction.
///
/// These instructions form a simple stack-based language used by the `SimpleExecutor`.
#[derive(Debug, Clone)]
pub enum Instr {
    /// Pushes a floating-point number onto the stack.
    Push(f64),
    /// Pops two numbers, adds them, and pushes the result.
    Add,
    /// Pops two numbers, subtracts the second from the first, and pushes the result.
    Sub,
    /// Pops two numbers, multiplies them, and pushes the result.
    Mul,
    /// Pops two numbers, divides the first by the second, and pushes the result.
    Div,
    /// Pops one number, negates it, and pushes the result.
    Neg,
}

/// Errors that can happen during compilation (AST -> bytecode)
#[derive(Debug)]
pub enum CompileError {
    /// An operator was encountered in the AST that is not supported by the bytecode compiler.
    UnsupportedOperator(String),
}

/// Implements the `Display` trait for `CompileError`, allowing it to be
/// easily formatted as a user-friendly string.
impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnsupportedOperator(s) => {
                write!(f, "unsupported operator during compilation: {}", s)
            }
        }
    }
}

impl Error for CompileError {}

/// Errors that can happen during execution of bytecode by the `SimpleExecutor`.
#[derive(Debug)]
pub enum ExecError {
    /// The executor attempted to pop a value from an empty stack.
    StackUnderflow { instr: String },
    /// A division by zero operation was attempted.
    DivisionByZero,
    /// The execution finished, but no result was left on the stack.
    NoResult,
    /// A generic execution error with a descriptive message.
    Other(String),
}

/// Implements the `Display` trait for `ExecError`, allowing it to be
/// easily formatted as a user-friendly string.
impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecError::StackUnderflow { instr } => {
                write!(f, "stack underflow while executing instruction '{}'", instr)
            }
            ExecError::DivisionByZero => write!(f, "division by zero"),
            ExecError::NoResult => write!(f, "execution finished with no result on the stack"),
            ExecError::Other(s) => write!(f, "execution error: {}", s),
        }
    }
}

impl Error for ExecError {}

/// Top-level evaluation error returned by the orchestrator (`evaluate_lines`).
///
/// This enum wraps errors from different stages of the interpretation pipeline
/// (parsing, compilation, execution) and provides additional context like
/// the original line content and absolute line number for better error reporting.
#[derive(Debug)]
pub enum EvalError {
    /// An error occurred during the parsing phase.
    /// Contains the `ParserError`, the original line string, and the original line offset.
    Parse(ParserError, String, usize),
    /// An error occurred during the bytecode compilation phase.
    /// Contains the `CompileError` and the input string that caused the error.
    Compile(CompileError, String),
    /// An error occurred during the bytecode execution phase.
    /// Contains the `ExecError` and the input string that caused the error.
    Exec(ExecError, String),
}

/// Implements the `Display` trait for `EvalError`, providing a comprehensive
/// and user-friendly error message that includes the error type, location,
/// and a snippet of the problematic code.
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Parse(e, line_str, original_line_offset) => {
                let (relative_line_num, col_num) = match e {
                    ParserError::UnexpectedToken { line, col, .. } => (*line, *col),
                    ParserError::UnexpectedEOF { line, col } => (*line, *col),
                    ParserError::InvalidNumber { line, col, .. } => (*line, *col),
                    ParserError::TokenizerError { line, col, .. } => (*line, *col),
                };

                // Calculate the absolute line number in the original file
                let absolute_line_num = original_line_offset + relative_line_num - 1; // -1 because relative_line_num is 1-based

                // The line_content is now the line_str itself, as the error is relative to it
                let line_content = line_str;
                let pointer = " ".repeat(col_num - 1) + "^";

                write!(
                    f,
                    "Error: {}\n{} | {}\n{} | {}",
                    e,
                    absolute_line_num,
                    line_content,
                    " ".repeat(absolute_line_num.to_string().len()),
                    pointer
                )
            }
            EvalError::Compile(e, input) => write!(f, "compile error: {} in input: {}", e, input),
            EvalError::Exec(e, input) => write!(f, "runtime error: {} in input: {}", e, input),
        }
    }
}

impl Error for EvalError {}

/// Compiler that lowers AST -> Vec<Instr>
///
/// This component takes an Abstract Syntax Tree (`Expr`) and translates it
/// into a sequence of bytecode instructions (`Instr`) that can be executed
/// by the `SimpleExecutor`.
pub struct BytecodeCompiler;

impl BytecodeCompiler {
    /// Compiles an expression into a vector of bytecode instructions.
    ///
    /// This function traverses the AST recursively, emitting appropriate
    /// bytecode instructions for each node.
    ///
    /// # Arguments
    /// * `expr` - A reference to the `Expr` (AST node) to compile.
    ///
    /// # Returns
    /// A `Result` which is `Ok` containing a `Vec<Instr>` on successful compilation,
    /// or `Err` containing a `CompileError` if an unsupported AST node is encountered.
    pub fn compile(expr: &Expr) -> Result<Vec<Instr>, CompileError> {
        let mut code = Vec::new();
        Self::compile_expr(expr, &mut code)?;
        Ok(code)
    }

    /// Recursively compiles an `Expr` node and appends its bytecode to the given vector.
    ///
    /// # Arguments
    /// * `expr` - A reference to the `Expr` (AST node) to compile.
    /// * `code` - A mutable reference to the vector where generated instructions will be appended.
    ///
    /// # Returns
    /// A `Result` which is `Ok(())` on successful compilation of the current node,
    /// or `Err` containing a `CompileError` if an unsupported operator is found.
    fn compile_expr(expr: &Expr, code: &mut Vec<Instr>) -> Result<(), CompileError> {
        match expr {
            Expr::Number(n) => {
                code.push(Instr::Push(*n));
                Ok(())
            }
            Expr::UnaryOp { op, expr: e } => {
                Self::compile_expr(e, code)?;
                match op {
                    TokenType::Minus => {
                        code.push(Instr::Neg);
                        Ok(())
                    }
                    TokenType::Plus => Ok(()), // no-op
                    other => Err(CompileError::UnsupportedOperator(format!("{:?}", other))),
                }
            }
            Expr::BinaryOp { left, op, right } => {
                Self::compile_expr(left, code)?;
                Self::compile_expr(right, code)?;
                match op {
                    TokenType::Plus => {
                        code.push(Instr::Add);
                        Ok(())
                    }
                    TokenType::Minus => {
                        code.push(Instr::Sub);
                        Ok(())
                    }
                    TokenType::Mul => {
                        code.push(Instr::Mul);
                        Ok(())
                    }
                    TokenType::Div => {
                        code.push(Instr::Div);
                        Ok(())
                    }
                    other => Err(CompileError::UnsupportedOperator(format!("{:?}", other))),
                }
            }
            Expr::Empty => {
                // Do nothing, produce no bytecode.
                Ok(())
            }
            Expr::EmptyParen => {
                code.push(Instr::Push(0.0));
                Ok(())
            }
        }
    }
}

/// A simple stack-based executor for `arith` bytecode.
///
/// This executor is stateless and evaluates a given slice of `Instr` (bytecode)
/// to produce a single floating-point result.
pub struct SimpleExecutor;

impl SimpleExecutor {
    /// Creates a new `SimpleExecutor`.
    pub fn new() -> Self {
        SimpleExecutor
    }

    /// Executes a sequence of bytecode instructions.
    ///
    /// The executor maintains an internal stack for intermediate calculations.
    ///
    /// # Arguments
    /// * `instructions` - A slice of `Instr` to be executed.
    ///
    /// # Returns
    /// A `Result` which is `Ok` containing the final `f64` result on successful execution,
    /// or `Err` containing an `ExecError` if a runtime error occurs (e.g., stack underflow, division by zero).
    pub fn execute(&self, instructions: &[Instr]) -> Result<f64, ExecError> {
        let mut stack: Vec<f64> = Vec::with_capacity(16);

        for instr in instructions {
            match instr {
                Instr::Push(n) => stack.push(*n),
                Instr::Add => {
                    let b = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Add".to_string(),
                    })?;
                    let a = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Add".to_string(),
                    })?;
                    stack.push(a + b);
                }
                Instr::Sub => {
                    let b = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Sub".to_string(),
                    })?;
                    let a = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Sub".to_string(),
                    })?;
                    stack.push(a - b);
                }
                Instr::Mul => {
                    let b = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Mul".to_string(),
                    })?;
                    let a = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Mul".to_string(),
                    })?;
                    stack.push(a * b);
                }
                Instr::Div => {
                    let b = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Div".to_string(),
                    })?;
                    if b == 0.0 {
                        return Err(ExecError::DivisionByZero);
                    }
                    let a = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Div".to_string(),
                    })?;
                    stack.push(a / b);
                }
                Instr::Neg => {
                    let a = stack.pop().ok_or(ExecError::StackUnderflow {
                        instr: "Neg".to_string(),
                    })?;
                    stack.push(-a);
                }
            }
        }

        match stack.pop() {
            Some(v) => Ok(v),
            None => Err(ExecError::NoResult),
        }
    }
}

/// Orchestrates the entire evaluation process for a multi-line input string.
///
/// This function handles line continuations (lines ending with `\`),
/// removes comments, tokenizes each logical expression, parses it into an AST,
/// compiles the AST into bytecode, and finally executes the bytecode.
///
/// It processes the input line by line, accumulating lines that end with a backslash
/// into a single logical expression. Each logical expression is then evaluated independently.
///
/// # Arguments
/// * `input` - The multi-line input string containing arithmetic expressions.
///
/// # Returns
/// A `Vec` of `Result`s, where each `Result` corresponds to the evaluation of one
/// logical expression. An `Ok` variant contains a tuple of the `f64` result and
/// the original expression string. An `Err` variant contains an `EvalError`
/// providing details about the error.
pub fn evaluate_lines(input: &str) -> Vec<Result<(f64, String), EvalError>> {
    let mut joined_expressions: Vec<(String, usize)> = Vec::new();
    let mut current_expression_buffer = String::new();
    let mut current_expression_start_line = 0;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_num = idx + 1; // 1-based line number

        // Remove comments
        let line_without_comment = raw_line.split(';').next().unwrap_or("").to_string();

        let trimmed_line_content = line_without_comment.trim(); // Trim all whitespace

        if current_expression_buffer.is_empty() {
            current_expression_start_line = line_num;
        }

        if trimmed_line_content.ends_with('\\') {
            // This line continues the expression
            current_expression_buffer
                .push_str(&trimmed_line_content[0..trimmed_line_content.len() - 1].trim());
            current_expression_buffer.push(' '); // Add a space for token separation
        } else {
            // This line completes an expression or is a single-line expression
            current_expression_buffer.push_str(trimmed_line_content); // Add the content of the current line

            // Only push if the accumulated expression is not empty or just whitespace
            if !current_expression_buffer.trim().is_empty() {
                joined_expressions.push((
                    current_expression_buffer.clone(),
                    current_expression_start_line,
                ));
            }
            current_expression_buffer.clear();
            current_expression_start_line = 0; // Reset
        }
    }

    // Handle any remaining accumulated expression if the file ends with a backslash
    // and it's not just whitespace
    if !current_expression_buffer.is_empty() && !current_expression_buffer.trim().is_empty() {
        joined_expressions.push((
            current_expression_buffer.clone(),
            current_expression_start_line,
        ));
    }

    // Evaluate each joined line separately
    let mut results = Vec::new();
    let executor = SimpleExecutor::new();

    for (line_str, original_line_offset) in joined_expressions {
        let trimmed = line_str.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tokens = match Tokenizer::new(trimmed.to_string()).tokenize() {
            Ok(tokens) => tokens,
            Err(TokenizerError::UnexpectedCharacter { found, line, col }) => {
                eprintln!("DEBUG: TokenizerError line = {}, col = {}", line, col);
                results.push(Err(EvalError::Parse(
                    ParserError::TokenizerError {
                        message: format!("Unexpected character '{}'", found),
                        line,
                        col,
                    },
                    line_str.to_string(), // Pass the specific line string
                    original_line_offset,
                )));
                continue;
            }
        };

        match Parser::new(tokens).parse() {
            Ok(ast) => match BytecodeCompiler::compile(&ast) {
                Ok(code) => {
                    if !code.is_empty() {
                        match executor.execute(&code) {
                            Ok(v) => results.push(Ok((v, line_str.to_string()))),
                            Err(e) => results.push(Err(EvalError::Exec(e, line_str.to_string()))),
                        }
                    }
                }
                Err(e) => results.push(Err(EvalError::Compile(e, line_str.to_string()))),
            },
            Err(e) => {
                results.push(Err(EvalError::Parse(
                    e,
                    line_str.to_string(),
                    original_line_offset,
                ))); // Pass the specific line string and offset
            }
        }
    }

    results
}
