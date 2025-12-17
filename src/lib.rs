use crate::checker::check::{Context, check_expr};
use crate::config::CompilerOptions;
use crate::syntax::parser::parse;
use crate::vm::core::VM;
use crate::vm::trans_ast_bytecode::Compiler;

pub mod checker;
pub mod compiler_errors;
pub mod config;
pub mod diagnostics;
pub mod syntax;
pub mod vm;

pub fn execute(source: &str, path: &str, compiler_options: &CompilerOptions) {
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errs) => {
            diagnostics::render_errors(source, path, errs);
            return;
        }
    };

    let ctx = Context::new();
    if let Err(err) = check_expr(&ctx, &ast) {
        diagnostics::render_errors(source, path, vec![err]);
        return;
    }

    if compiler_options.print_ast {
        println!(
            "AST: {}",
            ast.node.debug_ast(compiler_options.print_ast_indent_size)
        );
    }

    let mut compiler = Compiler::new();
    compiler.compile(&ast.node);

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
