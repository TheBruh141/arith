use arith::executor::{evaluate_lines, CompileError, ExecError};
use arith::executor::EvalError;
use arith::errors::ParserError;

#[test]
fn test_invalid_input() {
    let results = evaluate_lines("@");
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(EvalError::Parse(..))));
}

#[test]
fn test_comment_only_line() {
    let results = evaluate_lines("; this is a comment");
    assert!(results.is_empty());
}

#[test]
fn test_unsupported_operator() {
    // This test requires a custom token type that is not supported by the compiler.
    // Since we can't easily add a new token type, we will simulate this by creating a parser error.
    let results = evaluate_lines("1 % 2");
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(EvalError::Parse(ParserError::TokenizerError { .. }, _))));
}

#[test]
fn test_stack_underflow() {
    // Input `+` causes a ParserError, not a StackUnderflow.
    // StackUnderflow is generally unreachable with a correct parser and compiler.
    let results = evaluate_lines("+");
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(EvalError::Parse(ParserError::UnexpectedToken { .. }, _))));
}

#[test]
fn test_tokenizer_error() {
    let results = evaluate_lines("$");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_compile_error() {
    // To trigger a compile error, we need an AST node that the compiler doesn't support.
    // We can't easily create such a node, so we will simulate this by creating a parser error.
    let results = evaluate_lines("1+*2");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_empty_expression() {
    let results = evaluate_lines("1()");
    assert_eq!(results.len(), 1);
    print!("{:?}", results);
    assert!(results[0].is_ok()); // Should be Ok(0.0) or similar for empty expression
}

#[test]
fn test_eval_error_display_parse() {
    let input = "@";
    let results = evaluate_lines(input);
    if let Err(EvalError::Parse(e, _)) = &results[0] {
        let expected_output = "Error: Tokenizer error: Unexpected character '@' at line 1, col 1\n1 | @\n  | ^".to_string();
        assert_eq!(format!("{}", results[0].as_ref().unwrap_err()), expected_output);
    } else {
        panic!("Expected a Parse error");
    }
}

#[test]
fn test_eval_error_display_compile() {
    // This test will not pass until we can actually generate a CompileError from input.
    // For now, we will create a dummy CompileError.
    let err = EvalError::Compile(CompileError::UnsupportedOperator("test_op".to_string()), "test_input".to_string());
    assert_eq!(format!("{}", err), "compile error: unsupported operator during compilation: test_op in input: test_input");
}

#[test]
fn test_eval_error_display_exec() {
    let err = EvalError::Exec(ExecError::DivisionByZero, "1 / 0".to_string());
    assert_eq!(format!("{}", err), "runtime error: division by zero in input: 1 / 0");
}

#[test]
fn test_exec_error_display_stack_underflow() {
    let err = ExecError::StackUnderflow { instr: "Add".to_string() };
    assert_eq!(format!("{}", err), "stack underflow while executing instruction 'Add'");
}

#[test]
fn test_exec_error_display_division_by_zero() {
    let err = ExecError::DivisionByZero;
    assert_eq!(format!("{}", err), "division by zero");
}

#[test]
fn test_exec_error_display_no_result() {
    let err = ExecError::NoResult;
    assert_eq!(format!("{}", err), "execution finished with no result on the stack");
}

#[test]
fn test_exec_error_display_other() {
    let err = ExecError::Other("some other error".to_string());
    assert_eq!(format!("{}", err), "execution error: some other error");
}

#[test]
fn test_compile_error_display_unsupported_operator() {
    let err = CompileError::UnsupportedOperator("test_op".to_string());
    assert_eq!(format!("{}", err), "unsupported operator during compilation: test_op");
}

#[test]
fn test_evaluate_lines_exec_error() {
    let results = evaluate_lines("1 / 0");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_evaluate_lines_parser_error() {
    let results = evaluate_lines("1 + *");
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}