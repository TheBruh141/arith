#[cfg(test)]
mod tests {
    use arith::syntax::parser; // Import your parser function
    use chumsky::Parser;
    use arith::syntax::parser::parser;
    use arith::vm::trans_ast_bytecode::Compiler;
    use arith::vm::vm::{Value, VM};

    /// Helper function to compile and run a string of source code.
    /// Returns the final Value or panics if parsing/execution fails.
    fn run(source: &str) -> Value {
        let parser = parser();
        let ast = parser.parse(source).map_err(|e| {
            format!("Parse Error: {:?}", e)
        }).expect("Parsing failed");

        let mut compiler = Compiler::new();
        compiler.compile(&ast);

        // 3. Run
        let mut vm = VM::new(compiler.code);
        vm.debug_run().expect("VM Runtime Error")
    }

    // ========================================================================
    // 1. BASIC LITERALS AND ARITHMETIC
    // ========================================================================

    #[test]
    fn test_integers() {
        assert_eq!(run("123"), Value::Int(123));
        assert_eq!(run("0"), Value::Int(0));
    }

    #[test]
    fn test_booleans() {
        assert_eq!(run("true"), Value::Bool(true));
        assert_eq!(run("false"), Value::Bool(false));
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(run("1 + 2"), Value::Int(3));
        assert_eq!(run("10 - 4"), Value::Int(6));
        assert_eq!(run("3 * 4"), Value::Int(12));
        assert_eq!(run("20 / 5"), Value::Int(4));
    }

    // ========================================================================
    // 2. ORDER OF OPERATIONS (PRECEDENCE)
    // ========================================================================

    #[test]
    fn test_precedence() {
        // Multiplication (*) has higher precedence than Addition (+)
        // Should be 1 + (2 * 3) = 7, not (1 + 2) * 3 = 9
        assert_eq!(run("1 + 2 * 3"), Value::Int(7));

        // Parentheses override precedence
        assert_eq!(run("(1 + 2) * 3"), Value::Int(9));
    }

    #[test]
    fn test_associativity() {
        // Subtraction should be left-associative
        // (10 - 5) - 2 = 3, NOT 10 - (5 - 2) = 7
        assert_eq!(run("10 - 5 - 2"), Value::Int(3));

        // Division should be left-associative
        // (20 / 2) / 2 = 5, NOT 20 / (2 / 2) = 20
        assert_eq!(run("20 / 2 / 2"), Value::Int(5));
    }

    // ========================================================================
    // 3. CONTROL FLOW
    // ========================================================================

    #[test]
    fn test_if_expression() {
        assert_eq!(run("if true then 10 else 20"), Value::Int(10));
        assert_eq!(run("if false then 10 else 20"), Value::Int(20));
    }

    #[test]
    fn test_nested_if() {
        let src = "
            if false then 1
            else if true then 2
            else 3
        ";
        assert_eq!(run(src), Value::Int(2));
    }

    #[test]
    fn test_expression_inside_if() {
        // Conditions can be expressions, branches can be calculations
        println!("a");
        // assert_eq!(run("if 10 - 5 * 2 + 1 then 100 else 200"), Value::Int(100)); // 1 is treated as true if strict bool checks aren't enforced, else use true
        // If your language enforces strict bools in 'if':

        println!("a");
        // assert_eq!(run("if (1 + 1) * 5 - 10 then 1 else 0"), Value::Int(0)); // Assuming non-zero is true?
        // Or if strictly bool:
        println!("a");

        assert_eq!(run("if true then 1 + 1 else 2 * 2"), Value::Int(2));
    }

    // ========================================================================
    // 4. VARIABLES AND BINDING
    // ========================================================================

    #[test]
    fn test_let_binding() {
        let src = "let x = 10 in x + 5";
        assert_eq!(run(src), Value::Int(15));
    }

    #[test]
    fn test_nested_let() {
        let src = "
            let x = 10 in
            let y = 5 in
            x + y
        ";
        assert_eq!(run(src), Value::Int(15));
    }

    #[test]
    fn test_shadowing() {
        // Inner 'x' should shadow outer 'x'
        let src = "
            let x = 10 in
            let x = 20 in
            x
        ";
        assert_eq!(run(src), Value::Int(20));
    }

    #[test]
    fn test_shadowing_scope() {
        // The outer x should remain 10 after the inner let finishes
        // actually, in 'let x = ... in body', body is the scope.
        // This tests that the inner calculation uses the inner x.
        let src = "
            let x = 10 in
            (let x = 5 in x) + x
        ";
        // Inner `let x = 5 in x` evaluates to 5.
        // Outer `x` is still 10.
        // Result: 5 + 10 = 15.
        assert_eq!(run(src), Value::Int(15));
    }

    // ========================================================================
    // 5. FUNCTIONS AND CLOSURES
    // ========================================================================

    #[test]
    fn test_simple_lambda() {
        // Apply immediate lambda
        // Syntax: (.\ x : Int -> x + 1) 10
        let src = "( .\\ x : Int -> x + 1 ) 10";
        assert_eq!(run(src), Value::Int(11));
    }

    #[test]
    fn test_higher_order_types_syntax() {
        // Test that the parser handles complex types, even if the VM ignores them
        let src = "( .\\ x : Vector<Int, 10> -> 1 ) 0";
        assert_eq!(run(src), Value::Int(1));
    }

    #[test]
    fn test_closure_capture() {
        // THIS IS THE MOST IMPORTANT TEST FOR A FUNCTIONAL LANGUAGE
        // It tests if a function "remembers" the environment it was created in.

        let src = "
            let x = 10 in
            let addX = .\\ y : Int -> x + y in
            addX 5
        ";
        // `addX` captures `x = 10`. When called with 5, it should do 10 + 5.
        assert_eq!(run(src), Value::Int(15));
    }

    #[test]
    fn test_currying() {
        // Function that returns a function
        let src = "
            let makeAdder = .\\ x : Int -> (.\\ y : Int -> x + y) in
            let add5 = makeAdder 5 in
            add5 10
        ";
        assert_eq!(run(src), Value::Int(15));
    }

    #[test]
    fn test_function_as_argument() {
        // Passing a function to another function
        // apply = \f -> \x -> f x
        let src = "
            let apply = .\\ f : Int -> Int -> (.\\ val : Int -> f val) in
            let double = .\\ n : Int -> n * 2 in
            apply double 10
        ";
        assert_eq!(run(src), Value::Int(20));
    }

    // ========================================================================
    // 6. COMPLEX INTEGRATION
    // ========================================================================

    #[test]
    fn test_complex_integration() {
        // A mix of logic, math, and functions
        let src = "
            let a = 5 in
            let b = 10 in
            let calc = .\\ op : Bool ->
                       if op then a + b else a * b
            in
            (calc true) + (calc false)
        ";
        // calc true = 5 + 10 = 15
        // calc false = 5 * 10 = 50
        // Result = 15 + 50 = 65
        assert_eq!(run(src), Value::Int(65));
    }
}