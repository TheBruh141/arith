use crate::ast::{Expr, Statement};
use crate::errors::{ParserError, TokenizerError};
use crate::parser::Parser;
use crate::tokenizer::{TokenType, Tokenizer};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Instr {
    Push(f64),
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Load(String),
}

#[derive(Debug)]
pub enum CompileError {
    UnsupportedOperator(String),
    UndefinedVariable(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnsupportedOperator(s) => {
                write!(f, "unsupported operator during compilation: {}", s)
            }
            CompileError::UndefinedVariable(s) => {
                write!(f, "undefined variable: {}", s)
            }
        }
    }
}

impl Error for CompileError {}

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

#[derive(Debug)]
pub enum EvalError {
    Parse(ParserError, String, usize),
    Compile(CompileError, String),
    Exec(ExecError, String),
}

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

                let absolute_line_num = original_line_offset + relative_line_num - 1;

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

pub struct BytecodeCompiler;

impl BytecodeCompiler {
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
            Expr::Variable(name) => {
                code.push(Instr::Load(name.clone()));
                Ok(())
            }
            Expr::UnaryOp { op, expr: e } => {
                Self::compile_expr(e, code)?;
                match op {
                    TokenType::Minus => {
                        code.push(Instr::Neg);
                        Ok(())
                    }
                    TokenType::Plus => Ok(()),
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
            Expr::Empty => Ok(()),
            Expr::EmptyParen => {
                code.push(Instr::Push(0.0));
                Ok(())
            }
        }
    }
}

pub struct SimpleExecutor {
    symbols: HashMap<String, f64>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            symbols: HashMap::new(),
        }
    }

    pub fn execute(&mut self, instructions: &[Instr]) -> Result<f64, ExecError> {
        let mut stack: Vec<f64> = Vec::with_capacity(16);

        for instr in instructions {
            match instr {
                Instr::Push(n) => stack.push(*n),
                Instr::Load(name) => {
                    if let Some(value) = self.symbols.get(name) {
                        stack.push(*value);
                    } else {
                        return Err(ExecError::Other(format!("undefined variable: {}", name)));
                    }
                }
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

pub fn evaluate_lines(
    input: &str,
    executor: &mut SimpleExecutor,
) -> Vec<Result<(f64, String), EvalError>> {
    let mut joined_expressions: Vec<(String, usize)> = Vec::new();
    let mut current_expression_buffer = String::new();
    let mut current_expression_start_line = 0;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_num = idx + 1;

        let line_without_comment = raw_line.split(';').next().unwrap_or("").to_string();

        let trimmed_line_content = line_without_comment.trim();

        if current_expression_buffer.is_empty() {
            current_expression_start_line = line_num;
        }

        if trimmed_line_content.ends_with('\\') {
            current_expression_buffer
                .push_str(&trimmed_line_content[0..trimmed_line_content.len() - 1].trim());
            current_expression_buffer.push(' ');
        } else {
            current_expression_buffer.push_str(trimmed_line_content);

            if !current_expression_buffer.trim().is_empty() {
                joined_expressions.push((
                    current_expression_buffer.clone(),
                    current_expression_start_line,
                ));
            }
            current_expression_buffer.clear();
            current_expression_start_line = 0;
        }
    }

    if !current_expression_buffer.is_empty() && !current_expression_buffer.trim().is_empty() {
        joined_expressions.push((
            current_expression_buffer.clone(),
            current_expression_start_line,
        ));
    }

    let mut results = Vec::new();

    for (line_str, original_line_offset) in joined_expressions {
        let trimmed = line_str.trim();
        if trimmed.is_empty() {
            continue;
        }

        let tokens = match Tokenizer::new(trimmed.to_string()).tokenize() {
            Ok(tokens) => tokens,
            Err(TokenizerError::UnexpectedCharacter { found, line, col }) => {
                results.push(Err(EvalError::Parse(
                    ParserError::TokenizerError {
                        message: format!("Unexpected character '{}'", found),
                        line,
                        col,
                    },
                    line_str.to_string(),
                    original_line_offset,
                )));
                continue;
            }
        };

        match Parser::new(tokens).parse() {
            Ok(statement) => match statement {
                Statement::Expression(expr) => match BytecodeCompiler::compile(&expr) {
                    Ok(code) => {
                        if !code.is_empty() {
                            match executor.execute(&code) {
                                Ok(v) => results.push(Ok((v, line_str.to_string()))),
                                Err(e) => {
                                    results.push(Err(EvalError::Exec(e, line_str.to_string())))
                                }
                            }
                        }
                    }
                    Err(e) => results.push(Err(EvalError::Compile(e, line_str.to_string()))),
                },
                Statement::Let { name, value, .. } => match BytecodeCompiler::compile(&value) {
                    Ok(code) => {
                        if !code.is_empty() {
                            match executor.execute(&code) {
                                Ok(v) => {
                                    executor.symbols.insert(name, v);
                                }
                                Err(e) => {
                                    results.push(Err(EvalError::Exec(e, line_str.to_string())))
                                }
                            }
                        }
                    }
                    Err(e) => results.push(Err(EvalError::Compile(e, line_str.to_string()))),
                },
                Statement::Assignment { name, value } => {
                    if !executor.symbols.contains_key(&name) {
                        results.push(Err(EvalError::Exec(
                            ExecError::Other(format!("undefined variable: {}", name)),
                            line_str.to_string(),
                        )));
                        continue;
                    }
                    match BytecodeCompiler::compile(&value) {
                        Ok(code) => {
                            if !code.is_empty() {
                                match executor.execute(&code) {
                                    Ok(v) => {
                                        executor.symbols.insert(name, v);
                                    }
                                    Err(e) => {
                                        results.push(Err(EvalError::Exec(e, line_str.to_string())))
                                    }
                                }
                            }
                        }
                        Err(e) => results.push(Err(EvalError::Compile(e, line_str.to_string()))),
                    }
                }
                Statement::CompoundAssignment { name, op, value } => {
                    if !executor.symbols.contains_key(&name) {
                        results.push(Err(EvalError::Exec(
                            ExecError::Other(format!("undefined variable: {}", name)),
                            line_str.to_string(),
                        )));
                        continue;
                    }
                    match BytecodeCompiler::compile(&value) {
                        Ok(code) => {
                            if !code.is_empty() {
                                match executor.execute(&code) {
                                    Ok(v) => {
                                        let current_val = executor.symbols.get(&name).unwrap();
                                        let new_val = match op {
                                            TokenType::PlusAssign => *current_val + v,
                                            TokenType::MinusAssign => *current_val - v,
                                            TokenType::MulAssign => *current_val * v,
                                            TokenType::DivAssign => *current_val / v,
                                            _ => unreachable!(),
                                        };
                                        executor.symbols.insert(name, new_val);
                                    }
                                    Err(e) => {
                                        results.push(Err(EvalError::Exec(e, line_str.to_string())))
                                    }
                                }
                            }
                        }
                        Err(e) => results.push(Err(EvalError::Compile(e, line_str.to_string()))),
                    }
                }
            },
            Err(e) => {
                results.push(Err(EvalError::Parse(
                    e,
                    line_str.to_string(),
                    original_line_offset,
                )));
            }
        }
    }

    results
}
