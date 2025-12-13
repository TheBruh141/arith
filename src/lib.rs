use chumsky::Parser;
use crate::syntax::parser::parser;
use crate::vm::trans_ast_bytecode::Compiler;
use crate::vm::vm::VM;

pub mod checker;
pub mod syntax;
pub mod vm;


pub fn execute(source: &str) {
    // 1. Parse
    let parser = parser();
    let ast = match parser.parse(source) {
        Ok(ast) => ast,
        Err(errs) => {
            errs.into_iter().for_each(|e| println!("{:?}", e));
            return;
        }
    };

    println!("AST: {:?}", ast);

    // 2. Compile
    let mut compiler = Compiler::new();
    compiler.compile(&ast);
    println!("Bytecode: {:?}", compiler.code);

    // 3. Run VM
    let mut vm = VM::new(compiler.code);
    match vm.run() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("VM Error: {}", e),
    }
}
