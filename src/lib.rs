use crate::config::CompilerOptions;
use crate::syntax::parser::parser;
use crate::vm::trans_ast_bytecode::Compiler;
use crate::vm::vm::VM;
use chumsky::Parser;
use clap::{Args, FromArgMatches};

pub mod checker;
pub mod config;
pub mod syntax;
pub mod vm;

pub fn execute(source: &str, compiler_options: &CompilerOptions) {
    let parser = parser();
    let ast = match parser.parse(source) {
        Ok(ast) => ast,
        Err(errs) => {
            errs.into_iter().for_each(|e| println!("{:?}", e));
            return;
        }
    };

    if compiler_options.print_ast {
        println!("AST: {}", ast.debug_ast(compiler_options.print_ast_indent_size));
    }

    let mut compiler = Compiler::new();
    compiler.compile(&ast);

    if compiler_options.print_bytecode {
        println!("Bytecode: {:?}", compiler.code);

    }

    // 3. Run VM
    let mut vm = VM::new(compiler.code);
    match vm.run() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("VM Error: {}", e),
    }
}

