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
    print!(
        "reached11\
     "
    );

    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errs) => {
            let res: String = match &errs.ast {
                None => "unrecoverable AST".into(),
                Some(e) => e.node.debug_ast(compiler_options.print_ast_indent_size),
            };

            if compiler_options.print_ast {
                println!("Partial AST on Error: \n {res}",)
            }
            diagnostics::render_errors_from_partial_parse(source, path, errs);
            return;
        }
    };

    if compiler_options.print_ast {
        println!(
            "AST:\n {}",
            ast.node.debug_ast(compiler_options.print_ast_indent_size)
        );
    }
    let ctx = Context::new();
    if let Err(err) = check_expr(&ctx, &ast) {
        diagnostics::render_errors(source, path, vec![err]);
        return;
    }

    let mut compiler = Compiler::new();
    compiler.compile(&ast.node);

    // if compiler_options.print_bytecode {
    //     compiler.code
    // }

    // 3. Run VM
    let mut vm = VM::new(compiler.code);
    match vm.run() {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("VM Error: {}", e),
    }
}
