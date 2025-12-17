use crate::compiler_errors::CompileErr;
use ariadne::{Color, Fmt, Label, Report, ReportKind, sources};

pub fn render_errors(source: &str, path: &str, errs: Vec<CompileErr>) {
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
            CompileErr::NotAFunction { span, found_type } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message("Not a Function")
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(format!(
                        "This expression is of type '{}', which is not a function.",
                        found_type.debug_type()
                    ))
                    .with_color(Color::Red),
            ),
            CompileErr::BinaryOpMismatch {
                span,
                op,
                expected_operand_type,
                found_operand_type,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message(format!("Type Mismatch in Binary Operation '{}'", op))
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Expected operand of type '{}', but found '{}'",
                            expected_operand_type.debug_type(),
                            found_operand_type.debug_type()
                        ))
                        .with_color(Color::Red),
                ),
            CompileErr::IfConditionNotBool { span, found_type } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message("Invalid If Condition Type")
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(format!(
                        "The condition of an 'if' expression must be of type 'Bool', but found '{}'.",
                        found_type.debug_type()
                    ))
                    .with_color(Color::Red),
            ),
            CompileErr::IfBranchesMismatch {
                span,
                then_type,
                else_type,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message("If Branches Type Mismatch")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "The 'then' branch has type '{}' but the 'else' branch has type '{}'. Both branches must have the same type.",
                            then_type.debug_type(),
                            else_type.debug_type()
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
            CompileErr::Custom { span, message } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message(message.clone())
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(message.fg(Color::Red))
                    .with_color(Color::Red),
            ),
        };
        report
            .finish()
            .print(sources(vec![(path_string.clone(), source)]))
            .unwrap();
    });
}
