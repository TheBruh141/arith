use crate::compiler_errors::CompileErr;
use crate::syntax::ast::PartialParse;
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
            CompileErr::Generic { span, message } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message(&message)
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(message)
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
            CompileErr::UnknownType { span, type_name } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message(format!("Unknown Type '{}'", type_name))
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(format!("Type '{}' not found", type_name))
                    .with_color(Color::Red),
            ),
            CompileErr::UnknownField {
                span,
                struct_name,
                field_name,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message(format!(
                    "Unknown Field '{}' in struct '{}'",
                    field_name, struct_name
                ))
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Field '{}' does not exist in '{}'",
                            field_name, struct_name
                        ))
                        .with_color(Color::Red),
                ),
            CompileErr::MissingField {
                span,
                struct_name,
                field_name,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message(format!(
                    "Missing Field '{}' in initialization of '{}'",
                    field_name, struct_name
                ))
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!("Field '{}' is missing", field_name))
                        .with_color(Color::Red),
                ),
            CompileErr::NotAStruct { span, found } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
            .with_message("Expected a Struct for property access")
            .with_label(
                Label::new((path_string.clone(), span))
                    .with_message(format!(
                        "Expected a Struct but found type '{}'",
                        found.debug_type()
                    ))
                    .with_color(Color::Red),
            ),
            CompileErr::UnknownVariant {
                span,
                enum_name,
                variant_name,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message("Unknown Enum Variant")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Variant '{}' does not exist in enum '{}'",
                            variant_name, enum_name
                        ))
                        .with_color(Color::Red),
                ),

            CompileErr::ArityMismatch {
                span,
                expected,
                found,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message("Arity Mismatch")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Expected {} arguments, but found {}",
                            expected, found
                        ))
                        .with_color(Color::Red),
                ),

            CompileErr::NotAnEnum { span, found } => Report::build(
                ReportKind::Error,
                (path_string.clone(), span.clone()),
            )
                .with_message("Not an Enum")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Expected an enum value, but found '{}'",
                            found.debug_type()
                        ))
                        .with_color(Color::Red),
                ),

            CompileErr::PatternMismatch {
                span,
                expected,
                found,
            } => Report::build(ReportKind::Error, (path_string.clone(), span.clone()))
                .with_message("Pattern Mismatch")
                .with_label(
                    Label::new((path_string.clone(), span))
                        .with_message(format!(
                            "Pattern does not match expected type '{}': found {}",
                            expected.debug_type(),
                            found
                        ))
                        .with_color(Color::Red),
                ),

        };

        report
            .finish()
            .print(sources(vec![(path_string.clone(), source)]))
            .unwrap();
    });
}

pub fn render_errors_from_partial_parse(source: &str, path: &str, partial_parse: PartialParse) {
    render_errors(source, path, partial_parse.errors)
}
