use crate::checker::check::{check_expr, Context};
use crate::compiler_errors::CompileErr;
use crate::config::CompilerOptions;
use crate::syntax::parser::parse;
use crate::vm::trans_ast_bytecode::Compiler;
use crate::vm::vm::VM;
use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};

pub mod checker;
pub mod compiler_errors;
pub mod config;
pub mod syntax;
pub mod vm;

fn render_errors(source: &str, path: &str, errs: Vec<CompileErr>) {
    let path_string = path.to_string();
    errs.into_iter().for_each(|e| {
        let report = match e {
            CompileErr::UnexpectedToken {
                span,
                expected,
                found,
            } => {
                let mut report = Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                    .with_message("Unexpected token");
                report = report.with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Unexpected token {}",
                            found.as_deref().unwrap_or("end of file").fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                );
                if !expected.is_empty() {
                    report = report.with_note(format!(
                        "Expected one of: {}",
                        expected
                            .into_iter()
                            .map(|o| o.unwrap_or("end of file".to_string()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                report
            }
            CompileErr::UnclosedDelimiter { span, delimiter } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message(format!("Unclosed delimiter {}", delimiter))
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(format!("Unclosed delimiter '{}'", delimiter))
                    .with_color(Color::Red),
            ),
            CompileErr::TypeMismatch {
                span,
                expected,
                found,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message("Type Mismatch")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Expected type '{}', but found type '{}'",
                            expected.debug_type(),
                            found.debug_type()
                        ))
                        .with_color(Color::Red),
                ),
            CompileErr::UnknownVar { span, name } => {
                Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                    .with_message("Unknown Variable")
                    .with_label(
                        Label::new((path_string.clone(), span))
                            .with_message(format!("Variable '{}' not found in scope", name))
                            .with_color(Color::Red),
                    )
            }
            CompileErr::Custom(msg) => Report::build(ReportKind::Error, (path_string.clone(), 0..0)).with_message(msg),
        };
        report
            .finish()
            .print(sources(vec![(path_string.clone(), source)]))
            .unwrap();
    });
}

pub fn execute(source: &str, path: &str, compiler_options: &CompilerOptions) {
    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(errs) => {
            render_errors(source, path, errs);
            return;
        }
    };
    
    let ctx = Context::new();
    if let Err(err) = check_expr(&ctx, &ast) {
        render_errors(source, path, vec![err]);
        return;
    }

    if compiler_options.print_ast {
        println!(
            "AST: {}",
            ast.node
                .debug_ast(compiler_options.print_ast_indent_size)
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
