use std::error::Error;
use std::fmt;
use crate::errors::{ParserError, TokenizerError};
use crate::parser::{Expr, Parser};
use crate::tokenizer::{TokenType, Tokenizer};

/// Bytecode instructions
#[derive(Debug, Clone)]
pub enum Instr {
    Push(f64),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
}

/// Errors that can happen during compilation (AST -> bytecode)
#[derive(Debug)]
pub enum CompileError {
    UnsupportedOperator(String),
}

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

/// Errors that can happen during execution of bytecode
#[derive(Debug)]
pub enum ExecError {
    StackUnderflow { instr: String },
    DivisionByZero,
    NoResult,
    Other(String),
}

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

/// Top-level evaluation error returned by the orchestrator
#[derive(Debug)]
pub enum EvalError {
    Parse(ParserError, String),
    Compile(CompileError, String),
    Exec(ExecError, String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Parse(e, input) => {
                let (line_num, col_num) = match e {
                    ParserError::UnexpectedToken { line, col, .. } => (*line, *col),
                    ParserError::UnexpectedEOF { line, col } => (*line, *col),
                    ParserError::InvalidNumber { line, col, .. } => (*line, *col),
                    ParserError::TokenizerError { line, col, .. } => (*line, *col),
                };

                // Debug print line_num and col_num
                eprintln!("DEBUG: line_num = {}, col_num = {}", line_num, col_num);

                let line_content = input.lines().nth(line_num - 1).unwrap_or("");
                let pointer = " ".repeat(col_num - 1) + "^";

                write!(
                    f,
                    "Error: {}\n{} | {}\n{} | {}",
                    e,
                    line_num,
                    line_content,
                    " ".repeat(line_num.to_string().len()),
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
pub struct BytecodeCompiler;

impl BytecodeCompiler {
    /// Compile an expression into bytecode. Returns CompileError if some AST node can't be compiled.
    pub fn compile(expr: &Expr) -> Result<Vec<Instr>, CompileError> {
        let mut code = Vec::new();
        Self::compile_expr(expr, &mut code)?;
        Ok(code)
    }

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

/// Simple stack-based executor. Stateless: execute a slice of instructions and return a result.
pub struct SimpleExecutor;

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor
    }

    /// Execute instructions and return top-of-stack result or ExecError.
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

/// Orchestrator: preprocesses backslash continuations and evaluates each non-empty line.
/// Returns Vec<Result<f64, EvalError>> where each entry corresponds to a line's evaluation.
pub fn evaluate_lines(input: &str) -> Vec<Result<f64, EvalError>> {
    // Preprocess: join lines that end with backslash `\`
    let mut joined_lines: Vec<String> = Vec::new();
    let mut acc = String::new();

    for raw_line in input.lines() {
        let mut line = raw_line.trim_end().to_string();
        if line.ends_with('\\') {
            // remove trailing backslash and append (keep a space to separate tokens)
            line.pop(); // remove '\'
            if !acc.is_empty() {
                acc.push(' ');
            }
            acc.push_str(line.trim());
            // continue accumulating
        } else {
            if !acc.is_empty() {
                // we had accumulated lines
                if !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push_str(line.trim());
                joined_lines.push(acc.clone());
                acc.clear();
            } else {
                joined_lines.push(line.trim().to_string());
            }
        }
    }

    // If acc still has something (last line ended with backslash but no following line),
    // we discard it as it's an incomplete expression.
    if !acc.is_empty() {
        // Optionally, you could return an error here for incomplete input.
        // For now, we just ignore it.
    }

    // Evaluate each joined line separately
    let mut results = Vec::new();
    let executor = SimpleExecutor::new();

    for line_str in joined_lines {
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
                    input.to_string(),
                )));
                continue;
            }
        };

        match Parser::new(tokens).parse() {
            Ok(ast) => {
                match BytecodeCompiler::compile(&ast) {
                    Ok(code) => {
                        if !code.is_empty() {
                            match executor.execute(&code) {
                                Ok(v) => results.push(Ok(v)),
                                Err(e) => results.push(Err(EvalError::Exec(e, input.to_string()))),
                            }
                        }
                    }
                    Err(e) => results.push(Err(EvalError::Compile(e, input.to_string()))),
                }
            }
            Err(e) => {
                results.push(Err(EvalError::Parse(e, input.to_string())));
            }
        }
    }

    results
}
