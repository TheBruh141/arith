use arith::syntax::parser::parser;
use arith::vm::trans_ast_bytecode::Compiler;
use arith::vm::vm::VM;
use chumsky::Parser;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// Gen: "1 + 2 * 3 / 4 + 1 + ..."
fn gen_arithmetic(ops: usize) -> String {
    let mut s = "1".to_string();
    for _ in 0..ops {
        s.push_str(" + 2 * 3 / 1");
    }
    s
}

// Gen: "let v = 0 in v + v + v ..."
// Stresses the HashMap lookup and String hashing.
fn gen_var_access(accesses: usize) -> String {
    let mut s = "let v = 1 in v".to_string();
    for _ in 0..accesses {
        s.push_str(" + v");
    }
    s
}

// Gen: "if true then (if false then 1 else 0) else 0 ..."
// Stresses JUMP and JUMP_IF_FALSE opcodes.
fn gen_branching(depth: usize) -> String {
    if depth == 0 {
        return "1".to_string();
    }
    // Alternating true/false to prevent simple branch prediction optimization in the CPU
    let cond = if depth % 2 == 0 { "true" } else { "false" };
    // If true, dive deeper, else return 1.
    if cond == "true" {
        format!("if {} then ({}) else 0", cond, gen_branching(depth - 1))
    } else {
        format!("if {} then 0 else ({})", cond, gen_branching(depth - 1))
    }
}

// Gen: f(f(f(f(x))))
// Stresses CallFrame creation, Argument Binding, and Return.
fn gen_call_chain(depth: usize) -> String {
    let mut s = "let f = .\\x:Int -> x + 1 in ".to_string();
    let mut calls = "1".to_string();
    for _ in 0..depth {
        calls = format!("f ({})", calls);
    }
    s.push_str(&calls);
    s
}

// Gen: Deeply nested closures.
// Stresses the `MakeClosure` opcode which copies the environment.
fn gen_closure_capture(depth: usize) -> String {
    let mut s = String::new();
    // Create 'depth' variables
    for i in 0..depth {
        s.push_str(&format!("let v{} = {} in ", i, i));
    }
    // Create a closure that uses the innermost variable,
    // forcing it to capture the environment.
    s.push_str(&format!("( .\\x:Int -> x + v{} ) 1", depth - 1));
    s
}

// ============================================================================
// BENCHMARKS
// ============================================================================

pub fn feature_benchmarks(c: &mut Criterion) {
    let parser = parser();

    // 1. ALU THROUGHPUT (Math)
    // Measures raw VM loop speed and stack push/pop.
    let mut group = c.benchmark_group("Micro_ALU");
    for size in [100, 1000].iter() {
        let src = gen_arithmetic(*size);
        let ast = parser.parse(&*src).unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast);
        let code = compiler.code;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut vm = VM::new(code.clone());
                vm.run().unwrap()
            })
        });
    }
    group.finish();

    // 2. MEMORY / VARIABLE ACCESS
    // Measures the cost of `OpCode::Load` (HashMap lookup + String hashing).
    let mut group = c.benchmark_group("Micro_VarLookup");
    for size in [100, 1000].iter() {
        let src = gen_var_access(*size);
        let ast = parser.parse(&*src).unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast);
        let code = compiler.code;

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut vm = VM::new(code.clone());
                vm.run().unwrap()
            })
        });
    }
    group.finish();

    // 3. BRANCH PREDICTION
    // Measures `OpCode::Jump` and `OpCode::JumpIfFalse`.
    let mut group = c.benchmark_group("Micro_Branching");
    for depth in [10, 50].iter() {
        let src = gen_branching(*depth);
        let ast = parser.parse(&*src).unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast);
        let code = compiler.code;

        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, _| {
            b.iter(|| {
                let mut vm = VM::new(code.clone());
                vm.run().unwrap()
            })
        });
    }
    group.finish();

    // 4. STACK / FUNCTION CALLS
    // Measures `OpCode::Call` overhead (Frame allocation).
    let mut group = c.benchmark_group("Macro_FunctionCalls");
    for depth in [10, 100].iter() {
        let src = gen_call_chain(*depth);
        let ast = parser.parse(&*src).unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast);
        let code = compiler.code;

        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, _| {
            b.iter(|| {
                let mut vm = VM::new(code.clone());
                vm.run().unwrap()
            })
        });
    }
    group.finish();

    // 5. HEAP / CLOSURE ALLOCATION
    // Measures `OpCode::MakeClosure` overhead (Environment copying).
    let mut group = c.benchmark_group("Macro_ClosureCapture");
    for vars in [10, 50].iter() {
        let src = gen_closure_capture(*vars);
        let ast = parser.parse(&*src).unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(&ast);
        let code = compiler.code;

        group.bench_with_input(BenchmarkId::from_parameter(vars), vars, |b, _| {
            b.iter(|| {
                let mut vm = VM::new(code.clone());
                vm.run().unwrap()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, feature_benchmarks);
criterion_main!(benches);
