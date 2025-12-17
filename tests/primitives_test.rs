#[cfg(test)]
mod tests {
    use arith::syntax::parser::{parse, parser};
    use arith::vm::core::{VM, Value};
    use arith::vm::trans_ast_bytecode::Compiler;
    use chumsky::Parser;
    use num_bigint::BigInt;

    fn val_int(n: i64) -> Value {
        Value::Int(BigInt::from(n))
    }

    fn run(source: &str) -> Value {
        let _parser = parser();
        let ast = parse(source)
            .map_err(|e| format!("Parse Error: {:?}", e))
            .expect("Parsing failed");

        let mut compiler = Compiler::new();
        // Since we are testing primitives, ensure type checking passes?
        // The VM doesn't run type checker automatically in this test harness unless we call it.
        // But the compiler compiles AST.
        // To be thorough, we *could* run type checker here, but VM tests generally test runtime.
        // We will assume valid AST for now, or check type errors in a separate checker test.

        compiler.compile(&ast.node);
        let mut vm = VM::new(compiler.code);
        vm.debug_run().expect("VM Runtime Error")
    }

    #[test]
    fn test_u8_literals_and_arithmetic() {
        assert_eq!(run("10u8"), Value::U8(10));
        assert_eq!(run("255u8"), Value::U8(255));
        assert_eq!(run("10u8 + 20u8"), Value::U8(30));
        assert_eq!(run("255u8 + 1u8"), Value::U8(0)); // Wrapping
    }

    #[test]
    fn test_i32_literals_and_arithmetic() {
        assert_eq!(run("100i32"), Value::I32(100));
        assert_eq!(run("10i32 - 20i32"), Value::I32(-10));
        assert_eq!(run("2147483647i32"), Value::I32(i32::MAX));
    }

    #[test]
    fn test_f64_literals_and_arithmetic() {
        // Floating point comparisons are tricky, but exact literals should work for simple cases
        assert_eq!(run("1.5f64"), Value::F64(1.5));
        assert_eq!(run("1.5f64 + 2.5f64"), Value::F64(4.0));
        assert_eq!(run("10.0f64 / 2.0f64"), Value::F64(5.0));
    }

    #[test]
    fn test_usize_literal() {
        assert_eq!(run("100usize"), Value::Usize(100));
    }

    // Type mismatch at runtime (since we skip checker in this harness)
    #[test]
    #[should_panic(expected = "Type mismatch")]
    fn test_runtime_mixed_type_error() {
        // VM should panic or return Err if types differ and we didn't check them.
        // Compiling "1u8 + 2u32" -> PushU8(1), PushU32(2), Add.
        // VM Add: match (Values::U8, Value::U32) => None -> Error.
        run("1u8 + 2u32");
    }
}
