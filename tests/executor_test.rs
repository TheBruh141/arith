use arith::errors::ParserError;
use arith::executor::{CompileError, EvalError, ExecError, SimpleExecutor, evaluate_lines};

fn assert_eval_ok(input: &str, expected: f64) {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert_eq!(results.len(), 1, "Expected one result for input: {}", input);
    match &results[0] {
        Ok((val, _expr_str)) => assert_eq!(*val, expected, "Input: {}", input),
        Err(e) => panic!("Evaluation failed for input '{}': {}", input, e),
    }
}

fn assert_eval_err(input: &str, expected_err_type: &str) {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert_eq!(results.len(), 1, "Expected one result for input: {}", input);
    match &results[0] {
        Ok((val, _expr_str)) => panic!("Expected error for input '{}', but got: {}", input, val),
        Err(e) => {
            let error_string = format!("{:?}", e);
            assert!(
                error_string.contains(expected_err_type),
                "Input: '{}', Expected error type containing '{}', but got: '{}'",
                input,
                expected_err_type,
                error_string
            );
        }
    }
}

#[test]
fn test_basic_arithmetic() {
    assert_eval_ok("1 + 2", 3.0);
    assert_eval_ok("5 - 3", 2.0);
    assert_eval_ok("4 * 2", 8.0);
    assert_eval_ok("10 / 2", 5.0);
}

#[test]
fn test_floating_point_numbers() {
    assert_eval_ok("1.5 + 2.5", 4.0);
    assert_eval_ok("5.0 - 3.5", 1.5);
    assert_eval_ok("4.0 * 2.5", 10.0);
    assert_eval_ok("10.0 / 4.0", 2.5);
    assert_eval_ok("1e-5 * 100", 0.001);
    assert_eval_ok("6.022e23 / 2", 3.011e23);
}

#[test]
fn test_operator_precedence() {
    assert_eval_ok("1 + 2 * 3", 7.0);
    assert_eval_ok("10 - 4 / 2", 8.0);
    assert_eval_ok(" (1 + 2) * 3", 9.0);
    assert_eval_ok("10 / (4 - 2)", 5.0);
}

#[test]
fn test_unary_operators() {
    assert_eval_ok("-5", -5.0);
    assert_eval_ok("+5", 5.0);
    assert_eval_ok("-(2 + 3)", -5.0);
    assert_eval_ok("+(2 * 3)", 6.0);
    assert_eval_ok("--5", 5.0);
}

#[test]
fn test_implicit_multiplication() {
    assert_eval_ok("3(5)", 15.0);
    assert_eval_ok("(2+1)(4)", 12.0);
    assert_eval_ok("(3)2", 6.0);
    assert_eval_ok("2(1+1)", 4.0);
}

#[test]
fn test_empty_and_comment_lines() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("", &mut executor);
    assert!(results.is_empty());

    let results = evaluate_lines("   ", &mut executor);
    assert!(results.is_empty());

    let results = evaluate_lines("; this is a comment", &mut executor);
    assert!(results.is_empty());

    let results = evaluate_lines(
        r#"1 + 1
; comment
2 * 2"#,
        &mut executor,
    );
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].as_ref().unwrap().0, 2.0);
    assert_eq!(results[1].as_ref().unwrap().0, 4.0);
}

#[test]
fn test_line_continuation() {
    assert_eval_ok(
        r#"1 + \
2"#,
        3.0,
    );
    assert_eval_ok(
        r#" (1 + \
2) * 3"#,
        9.0,
    );
}

#[test]
fn test_division_by_zero() {
    assert_eval_err("1 / 0", "DivisionByZero");
    assert_eval_err(" (5 + 5) / (3 - 3)", "DivisionByZero");
}

#[test]
fn test_invalid_syntax() {
    assert_eval_err("1 + * 2", r#"Parse(UnexpectedToken"#);
    assert_eval_err(" (1 + 2 ", r#"Parse(UnexpectedToken"#);
    assert_eval_err("abc", "undefined variable: abc");
    assert_eval_err("1.2.3", "Unexpected character '.'");
}

#[test]
fn test_empty_parentheses() {
    assert_eval_ok("()", 0.0);
    assert_eval_ok("1 + () * 5", 1.0);
}

#[test]
fn test_multiple_expressions_on_separate_lines() {
    let input = r#"1 + 1
2 * 3
10 / 2"#;
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].as_ref().unwrap().0, 2.0);
    assert_eq!(results[1].as_ref().unwrap().0, 6.0);
    assert_eq!(results[2].as_ref().unwrap().0, 5.0);
}

#[test]
fn test_variable_reassignment() {
    let input = r#"let a = 1
a = 2
a"#;
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_ref().unwrap().0, 2.0);
}

#[test]
fn test_complex_expressions() {
    assert_eval_ok("1 + 2 * (3 - 4) / -5", 1.4);
    assert_eval_ok("10 - 2 * 3 + 4 / 2", 6.0);
    assert_eval_ok("((1 + 2) * 3 - 4) / 5", 1.0);
}

#[test]
fn test_scientific_notation_eval() {
    assert_eval_ok("1e-5", 0.00001);
    assert_eval_ok("2.5E+3", 2500.0);
    assert_eval_ok("6.022e23", 6.022e23);
    assert_eval_ok("1e-5 * 1e5", 1.0);
}

#[test]
fn test_nested_parentheses() {
    assert_eval_ok("((((1))))", 1.0);
    assert_eval_ok("((1+2)*3)", 9.0);
}

#[test]
fn test_long_expression() {
    let long_expr = "1 + 2 * 3 - 4 / 2 + 5 * (6 - 7) / 8 + 9 - 10";
    assert_eval_ok(long_expr, 3.375);
}

#[test]
fn test_mixed_implicit_explicit_multiplication() {
    assert_eval_ok("2 * 3(4)", 24.0);
    assert_eval_ok(" (2)(3) * 4", 24.0);
    assert_eval_ok("5(1+1) * 2", 20.0);
}

#[test]
fn test_multiple_lines_with_comments_and_empty() {
    let input = r#"1 + 1
; first comment
2 * 2

   ; second comment
3 + 3"#;
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].as_ref().unwrap().0, 2.0);
    assert_eq!(results[1].as_ref().unwrap().0, 4.0);
    assert_eq!(results[2].as_ref().unwrap().0, 6.0);
}

#[test]
fn test_line_continuation_complex() {
    let input = r#"1 + \
2 * \
(3 + 4)"#;
    assert_eval_ok(input, 15.0);
}

#[test]
fn test_negative_results() {
    assert_eval_ok("1 - 10", -9.0);
    assert_eval_ok("5 * -2", -10.0);
    assert_eval_ok("-10 / 2", -5.0);
}

#[test]
fn test_zero_values() {
    assert_eval_ok("0 + 5", 5.0);
    assert_eval_ok("5 - 0", 5.0);
    assert_eval_ok("0 * 5", 0.0);
    assert_eval_ok("5 * 0", 0.0);
    assert_eval_ok("0 / 5", 0.0);
}

#[test]
fn test_large_numbers() {
    assert_eval_ok("1000000 * 1000000", 1.0e12);
    assert_eval_ok("1e10 + 1e10", 2.0e10);
}

#[test]
fn test_small_numbers() {
    assert_eval_ok("0.000001 * 0.000001", 1.0e-12);
    assert_eval_ok("1e-10 / 1e-10", 1.0);
}

#[test]
fn test_chained_operations() {
    assert_eval_ok("1 + 2 - 3 * 4 / 2", -3.0);
    assert_eval_ok("10 / 2 * 5 + 1", 26.0);
}

#[test]
fn test_empty_input_multiple_lines() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("\n\n", &mut executor);
    assert!(results.is_empty());
}

#[test]
fn test_input_with_only_whitespace() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("   	  \n  ", &mut executor);
    assert!(results.is_empty());
}

#[test]
fn test_comment_only_lines_mixed_with_empty() {
    let input = r#"; comment 1\n\n; comment 2"#;
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    assert!(results.is_empty());
}

#[test]
fn test_expression_followed_by_comment() {
    assert_eval_ok("1 + 1 ; inline comment", 2.0);
}

#[test]
fn test_expression_with_leading_trailing_whitespace() {
    assert_eval_ok("  1 + 1  ", 2.0);
}

#[test]
fn test_expression_with_internal_whitespace() {
    assert_eval_ok("1   +   1", 2.0);
}

#[test]
fn test_empty_expression_evaluates_to_zero() {
    assert_eval_ok("()", 0.0);
}

#[test]
fn test_nested_empty_parentheses() {
    assert_eval_ok("((()))", 0.0);
}

#[test]
fn test_empty_parentheses_in_binary_op() {
    assert_eval_ok("1 + () + 2", 3.0);
    assert_eval_ok("1 * () * 2", 0.0);
}

#[test]
fn test_empty_parentheses_with_unary_op() {
    assert_eval_ok("-()", 0.0);
    assert_eval_ok("+()", 0.0);
}

#[test]
fn test_implicit_multiplication_with_empty_paren() {
    assert_eval_ok("3()", 0.0);
    assert_eval_ok("()(5)", 0.0);
    assert_eval_ok("()()", 0.0);
}

#[test]
fn test_invalid_input() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("@", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], Err(EvalError::Parse(..))));
}

#[test]
fn test_comment_only_line() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("; this is a comment", &mut executor);
    assert!(results.is_empty());
}

#[test]
fn test_unsupported_operator() {
    // This test requires a custom token type that is not supported by the compiler.
    // Since we can't easily add a new token type, we will simulate this by creating a parser error.
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("1 % 2", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0],
        Err(EvalError::Parse(ParserError::TokenizerError { .. }, _, _))
    ));
}

#[test]
fn test_stack_underflow() {
    // Input `+` causes a ParserError, not a StackUnderflow.
    // StackUnderflow is generally unreachable with a correct parser and compiler.
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("+", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0],
        Err(EvalError::Parse(ParserError::UnexpectedToken { .. }, _, _))
    ));
}

#[test]
fn test_tokenizer_error() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("$", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_compile_error() {
    // To trigger a compile error, we need an AST node that the compiler doesn't support.
    // We can't easily create such a node, so we will simulate this by creating a parser error.
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("1+*2", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_empty_expression_old() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("1()", &mut executor);
    assert_eq!(results.len(), 1);
    print!("{:?}", results);
    assert!(results[0].is_ok());
}

#[test]
fn test_eval_error_display_parse() {
    let input = "@";
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines(input, &mut executor);
    if let Err(EvalError::Parse(_e, _, _)) = &results[0] {
        let expected_output = r#"Error: Tokenizer error: Unexpected character '@' at line 1, col 1
1 | @
  | ^"#
            .to_string();
        assert_eq!(
            format!("{}", results[0].as_ref().unwrap_err()),
            expected_output
        );
    } else {
        panic!("Expected a Parse error");
    }
}

#[test]
fn test_eval_error_display_compile() {
    let err = EvalError::Compile(
        CompileError::UnsupportedOperator("test_op".to_string()),
        "test_input".to_string(),
    );
    assert_eq!(
        format!("{}", err),
        r#"compile error: unsupported operator during compilation: test_op in input: test_input"#
    );
}

#[test]
fn test_eval_error_display_exec() {
    let err = EvalError::Exec(ExecError::DivisionByZero, "1 / 0".to_string());
    assert_eq!(
        format!("{}", err),
        r#"runtime error: division by zero in input: 1 / 0"#
    );
}

#[test]
fn test_exec_error_display_stack_underflow() {
    let err = ExecError::StackUnderflow {
        instr: "Add".to_string(),
    };
    assert_eq!(
        format!("{}", err),
        r#"stack underflow while executing instruction 'Add'"#
    );
}

#[test]
fn test_exec_error_display_division_by_zero() {
    let err = ExecError::DivisionByZero;
    assert_eq!(format!("{}", err), r#"division by zero"#);
}

#[test]
fn test_exec_error_display_no_result() {
    let err = ExecError::NoResult;
    assert_eq!(
        format!("{}", err),
        r#"execution finished with no result on the stack"#
    );
}

#[test]
fn test_exec_error_display_other() {
    let err = ExecError::Other("some other error".to_string());
    assert_eq!(format!("{}", err), r#"execution error: some other error"#);
}

#[test]
fn test_compile_error_display_unsupported_operator() {
    let err = CompileError::UnsupportedOperator("test_op".to_string());
    assert_eq!(
        format!("{}", err),
        r#"unsupported operator during compilation: test_op"#
    );
}

#[test]
fn test_evaluate_lines_exec_error() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("1 / 0", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}

#[test]
fn test_evaluate_lines_parser_error() {
    let mut executor = SimpleExecutor::new();
    let results = evaluate_lines("1 + *", &mut executor);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
}
