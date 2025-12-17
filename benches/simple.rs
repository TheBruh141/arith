use arith::syntax::parser::*;
use arith::vm::core::VM;
use arith::vm::trans_ast_bytecode::Compiler;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// Helper to generate a massive arithmetic string: "1 + 2 + 1 + 2 + ..."
fn generate_huge_arithmetic(n: usize) -> String {
    let mut s = "1".to_string();
    for _ in 0..n {
        s.push_str(" + 2");
    }
    s
}

// Helper to generate deeply nested lets: "let x = 1 in let x = x + 1 in ... x"
fn generate_deep_scope(n: usize) -> String {
    let mut s = "let x = 0 in ".to_string();
    for _ in 0..n {
        s.push_str("let x = x + 1 in ");
    }
    s.push_str("x");
    s
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let huge_math_source = generate_huge_arithmetic(1000);
    let deep_scope_source = generate_deep_scope(500);

    // -----------------------------------------------------------------------
    // 1. PARSER BENCHMARKS
    // -----------------------------------------------------------------------
    let mut group = c.benchmark_group("parser");
    group.sample_size(50); // Parsing is slow, reduce samples if needed

    group.bench_function("parse_arithmetic_1000_ops", |b| {
        b.iter(|| parse(black_box(huge_math_source.clone().as_str())).unwrap())
    });

    group.bench_function("parse_nested_scope_500_depth", |b| {
        b.iter(|| parse(black_box(deep_scope_source.clone().as_str())).unwrap())
    });
    group.finish();

    // -----------------------------------------------------------------------
    // 2. COMPILER BENCHMARKS
    // -----------------------------------------------------------------------
    let mut group = c.benchmark_group("compiler");

    // Pre-parse ASTs so we only measure compilation
    let math_ast = parse(huge_math_source.as_str()).unwrap();
    let scope_ast = parse(deep_scope_source.as_str()).unwrap();

    group.bench_function("compile_arithmetic", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(&math_ast.node));
            compiler.code
        })
    });

    group.bench_function("compile_scope", |b| {
        b.iter(|| {
            let mut compiler = Compiler::new();
            compiler.compile(black_box(&scope_ast.node));
            compiler.code
        })
    });
    group.finish();

    // -----------------------------------------------------------------------
    // 3. VM BENCHMARKS
    // -----------------------------------------------------------------------
    let mut group = c.benchmark_group("vm");

    // Pre-compile code so we only measure execution
    let mut c1 = Compiler::new();
    c1.compile(&math_ast.node);
    let math_code = c1.code;

    let mut c2 = Compiler::new();
    c2.compile(&scope_ast.node);
    let scope_code = c2.code;

    // Bench 1: Heavy Arithmetic (Stack push/pop/add)
    group.bench_function("vm_arithmetic_1000_ops", |b| {
        b.iter(|| {
            // We must clone the code for every run because the VM consumes/modifies state
            // (Note: In a read-only code VM, we wouldn't need to clone instructions,
            // but VM might modify instruction pointers or stack, I'm tired...)
            let mut vm = VM::new(math_code.clone());
            vm.run().unwrap()
        })
    });

    // Bench 2: Variable Lookup & Shadowing (HashMap & Vec operations)
    group.bench_function("vm_nested_scope_500_depth", |b| {
        b.iter(|| {
            let mut vm = VM::new(scope_code.clone());
            vm.run().unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
