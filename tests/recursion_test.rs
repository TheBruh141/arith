use arith::syntax::parser::parse;
use arith::vm::core::{VM, Value};
use arith::vm::trans_ast_bytecode::Compiler;
use num_bigint::BigInt;

fn run_vm(src: &str) -> Value {
    let parsed = parse(src).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(&parsed.node);
    let mut vm = VM::new(compiler.code);
    vm.run().expect("VM run failed")
}

#[test]
fn test_factorial_generic() {
    // Factorial: 5! = 120
    let src = r#"
    let rec factorial = .\n: Int ->
        if n < 2 then 1 else n * factorial (n - 1)
    in factorial 5
    "#;
    let res = run_vm(src);
    assert_eq!(res, Value::Int(BigInt::from(120)));
}

#[test]
fn test_fibonacci() {
    // 6th fib: 0 1 1 2 3 5 8
    // fib(0)=0, fib(1)=1, fib(2)=1, fib(6)=8
    let src = r#"
    let rec fib = .\n: Int ->
        if n == 0 then 0
        else if n == 1 then 1
        else fib(n - 1) + fib(n - 2)
    in fib 6
    "#;
    let res = run_vm(src);
    assert_eq!(res, Value::Int(BigInt::from(8)));
}

#[test]
fn test_mutually_shadowed_recursion() {
    // Ensure inner let rec shadows outer
    let src = r#"
    let rec f = .\x: Int -> x + 1 in
    let rec f = .\x: Int -> x * 2 in
    f 10
    "#;
    let res = run_vm(src);
    // Should use the inner one: 10 * 2 = 20
    assert_eq!(res, Value::Int(BigInt::from(20)));
}
