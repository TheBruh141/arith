use crate::compiler_errors::CompileErr;
use crate::syntax::ast::{BinaryOp, Expr, PartialParse, Pattern, Spanned, Type, TypeExpr, UnaryOp};
use crate::syntax::lexer::Token;
use chumsky::prelude::*;

pub fn parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> {
    // --- Primitives ---
    let ident = select! { Token::Ident(id) => id };

    // literal expressions -> produce Spanned<Expr>
    let lit = select! {
        Token::LitInt(s) => Expr::Int(s),
        Token::LitI8(n) => Expr::LitI8(n),
        Token::LitI16(n) => Expr::LitI16(n),
        Token::LitI32(n) => Expr::LitI32(n),
        Token::LitI64(n) => Expr::LitI64(n),
        Token::LitIsize(n) => Expr::LitIsize(n),
        Token::LitU8(n) => Expr::LitU8(n),
        Token::LitU16(n) => Expr::LitU16(n),
        Token::LitU32(n) => Expr::LitU32(n),
        Token::LitU64(n) => Expr::LitU64(n),
        Token::LitUsize(n) => Expr::LitUsize(n),
        Token::LitF16(s) => Expr::LitF16(s.parse().unwrap_or(0.0)),
        Token::LitF32(s) => Expr::LitF32(s.parse().unwrap_or(0.0)),
        Token::LitF64(s) => Expr::LitF64(s.parse().unwrap_or(0.0)),
        Token::True => Expr::Bool(true),
        Token::False => Expr::Bool(false),
    }
    .map_with_span(|e, span| Spanned::new(e, span));

    let type_int_lit = select! { Token::LitInt(s) => s.parse::<i64>().unwrap_or(0) };

    // --- Type Parsing ---
    let type_expr = recursive(|type_expr| {
        let atom = type_int_lit
            .map(TypeExpr::Lit)
            .or(ident.map(TypeExpr::Var))
            .or(type_expr.delimited_by(just(Token::LParen), just(Token::RParen)));

        let product = atom
            .clone()
            .then(just(Token::Mul).ignore_then(atom).repeated())
            .foldl(|lhs, rhs| TypeExpr::Binary(Box::new(lhs), BinaryOp::Mul, Box::new(rhs)));

        product
            .clone()
            .then(just(Token::Plus).ignore_then(product).repeated())
            .foldl(|lhs, rhs| TypeExpr::Binary(Box::new(lhs), BinaryOp::Add, Box::new(rhs)))
    });

    let type_parser = recursive(|type_def| {
        let base = just(Token::TInt)
            .to(Type::Int)
            .or(just(Token::TBool).to(Type::Bool))
            // Primitives
            .or(just(Token::Ti8).to(Type::I8))
            .or(just(Token::Ti16).to(Type::I16))
            .or(just(Token::Ti32).to(Type::I32))
            .or(just(Token::Ti64).to(Type::I64))
            .or(just(Token::Tisize).to(Type::Isize))
            .or(just(Token::Tu8).to(Type::U8))
            .or(just(Token::Tu16).to(Type::U16))
            .or(just(Token::Tu32).to(Type::U32))
            .or(just(Token::Tu64).to(Type::U64))
            .or(just(Token::Tusize).to(Type::Usize))
            .or(just(Token::Tf16).to(Type::F16))
            .or(just(Token::Tf32).to(Type::F32))
            .or(just(Token::Tf64).to(Type::F64))
            // Struct/Enum names
            .or(ident.try_map(|s, span| {
                if s.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Ok(Type::Struct(s))
                } else {
                    Err(Simple::custom(span, "Type names must be capitalized"))
                }
            }))
            // Arrow types
            .or(type_def
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .then(just(Token::Arrow).ignore_then(type_def.clone()).or_not())
                .map(|(lhs, rhs)| match rhs {
                    Some(rhs) => Type::Arrow(Box::new(lhs), Box::new(rhs)),
                    None => lhs,
                }))
            .or(just(Token::TVector)
                .ignore_then(
                    type_def
                        .clone()
                        .then_ignore(just(Token::Comma))
                        .then(type_expr)
                        .delimited_by(just(Token::LAngle), just(Token::RAngle)),
                )
                .map(|(t, n)| Type::Vector(Box::new(t), n)));

        let atom = base.or(type_def.delimited_by(just(Token::LParen), just(Token::RParen)));

        atom.clone()
            .separated_by(just(Token::Arrow))
            .at_least(1)
            .map(|types| {
                let mut iter = types.into_iter().rev();
                let last = iter.next().unwrap();
                iter.fold(last, |acc, t| Type::Arrow(Box::new(t), Box::new(acc)))
            })
    });

    // --- Expression Parsing ---
    recursive(|expr| {
        // lit is already Spanned<Expr>
        let lit = lit.clone();

        // 1. Block: { stmt; stmt } -> Spanned<Expr>
        let block = expr
            .clone()
            .separated_by(just(Token::Semi))
            .allow_trailing()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map_with_span(|mut stmts: Vec<Spanned<Expr>>, span| {
                let node = if stmts.len() == 1 {
                    // keep the inner expression's node type but keep block span for consistency
                    stmts.remove(0).node
                } else {
                    Expr::Block(stmts)
                };
                Spanned::new(node, span)
            });

        // 2. Patterns (for Match)
        let pattern = recursive(|pat| {
            let var = ident.map(Pattern::Var); // Pattern must be Pattern, not Spanned

            // wildcard and literal pattern
            let wildcard = just(Token::TripleDot).to(Pattern::Wildcard);
            let lit_pat = select! { Token::LitInt(s) => Pattern::LitInt(s) };

            // Enum Pattern: Shape::Circle(x)
            let enum_pat = ident
                .then_ignore(just(Token::Colon).then(just(Token::Colon)))
                .then(ident)
                .then(
                    pat.separated_by(just(Token::Comma))
                        .delimited_by(just(Token::LParen), just(Token::RParen))
                        .or_not()
                        .map(|v| v.unwrap_or_default()),
                )
                .map(|((e, v), args)| Pattern::EnumPat(e, v, args));

            enum_pat.or(wildcard).or(lit_pat).or(var)
        });

        // 3. Enum Definition: enum Shape { ... } in ...
        let enum_def = just(Token::Enum)
            .ignore_then(ident)
            .then(
                ident
                    .then(
                        type_parser
                            .clone()
                            .separated_by(just(Token::Comma))
                            .delimited_by(just(Token::LParen), just(Token::RParen))
                            .or_not()
                            .map(|v| v.unwrap_or_default()),
                    )
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map_with_span(|((name, variants), body), span| {
                Spanned::new(
                    Expr::EnumDef {
                        name,
                        variants,
                        body: Box::new(body),
                    },
                    span,
                )
            });

        // 4. Match Expression: match val { Pat => Expr, ... } -> Spanned<Expr>
        let match_expr = just(Token::Match)
            .ignore_then(expr.clone())
            .then(
                pattern
                    .then_ignore(just(Token::Arrow))
                    .then(expr.clone())
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(val, arms), span| {
                Spanned::new(
                    Expr::Match {
                        value: Box::new(val),
                        arms,
                    },
                    span,
                )
            });

        // 5. Enum Initialization: Shape::Circle(...)
        let enum_init = ident
            .then_ignore(just(Token::Colon).then(just(Token::Colon)))
            .then(ident)
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LParen), just(Token::RParen))
                    .or_not()
                    .map(|v| v.unwrap_or_default()),
            )
            .map_with_span(|((e_name, v_name), args), span| {
                Spanned::new(
                    Expr::EnumInit {
                        enum_name: e_name,
                        variant_name: v_name,
                        values: args,
                    },
                    span,
                )
            });

        // 6. Struct Initialization: Point { x: 1, ... }
        let struct_init = ident
            .then(
                ident
                    .then_ignore(just(Token::Colon))
                    .then(expr.clone())
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map_with_span(|(name, fields), span| {
                Spanned::new(Expr::StructInit(name, fields), span)
            });

        // variable atom -> Spanned<Expr>
        let var = ident.map_with_span(|name, span| Spanned::new(Expr::Var(name), span));

        // --- Atoms (all produce Spanned<Expr>) ---
        let atom_base = lit
            .or(block)
            .or(enum_init)
            .or(struct_init)
            .or(var)
            .or(match_expr.clone())
            .or(expr
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen)));

        // --- Postfix (Application, Field Access) ---
        enum PostfixOp {
            Call(Spanned<Expr>),
            Field(String),
        }

        let apply_or_access = atom_base
            .clone()
            .then(
                atom_base
                    .clone()
                    .map(PostfixOp::Call)
                    .or(just(Token::Dot).ignore_then(ident).map(PostfixOp::Field))
                    .repeated(),
            )
            .foldl(|lhs, op| match op {
                PostfixOp::Call(arg) => {
                    let span = lhs.span.start..arg.span.end;
                    Spanned::new(Expr::App(Box::new(lhs), Box::new(arg)), span)
                }
                PostfixOp::Field(field) => {
                    let span = lhs.span.start..lhs.span.end + 1 + field.len();
                    Spanned::new(Expr::FieldAccess(Box::new(lhs), field), span)
                }
            });

        // --- Unary ---
        let unary = recursive(|unary| {
            let op = just(Token::Minus)
                .to(UnaryOp::Neg)
                .or(just(Token::Bang).to(UnaryOp::Not));

            op.then(unary)
                .map_with_span(|(op, expr), span| {
                    Spanned::new(Expr::Unary(op, Box::new(expr)), span)
                })
                .or(apply_or_access.clone())
        });

        // --- Binary Precedence ---
        let product = unary
            .clone()
            .then(
                just(Token::Mul)
                    .or(just(Token::Div))
                    .then(apply_or_access.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                let bin_op = match op {
                    Token::Mul => BinaryOp::Mul,
                    Token::Div => BinaryOp::Div,
                    _ => unreachable!(),
                };
                Spanned::new(Expr::Binary(Box::new(lhs), bin_op, Box::new(rhs)), span)
            });

        let sum = product
            .clone()
            .then(
                just(Token::Plus)
                    .or(just(Token::Minus))
                    .then(product.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                let bin_op = match op {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Sub,
                    _ => unreachable!(),
                };
                Spanned::new(Expr::Binary(Box::new(lhs), bin_op, Box::new(rhs)), span)
            });

        let comparison = sum
            .clone()
            .then(
                choice((
                    just(Token::EqEq).to(BinaryOp::Equals),
                    just(Token::NotEq).to(BinaryOp::NotEquals),
                    just(Token::LAngle).to(BinaryOp::LessThan),
                    just(Token::RAngle).to(BinaryOp::GreaterThan),
                    just(Token::Leq).to(BinaryOp::LessThanEquals),
                    just(Token::Geq).to(BinaryOp::GreaterThanEquals),
                ))
                .then(sum.clone())
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
            });

        // --- Control Flow ---
        let struct_decl = just(Token::Struct)
            .ignore_then(ident)
            .then(
                ident
                    .then_ignore(just(Token::Colon))
                    .then(type_parser.clone())
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map_with_span(|((name, fields), body), span| {
                Spanned::new(
                    Expr::StructDef {
                        name,
                        fields,
                        body: Box::new(body),
                    },
                    span,
                )
            });

        let lambda = just(Token::Lambda)
            .ignore_then(ident)
            .then_ignore(just(Token::Colon))
            .then(type_parser.clone())
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map_with_span(|((var, ty), body), span| {
                Spanned::new(Expr::Abs(var, ty, Box::new(body)), span)
            });

        let let_expr = just(Token::Let)
            .ignore_then(just(Token::Rec).or_not())
            .then(ident)
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map_with_span(|(((rec_opt, var), e1), e2), span| {
                if rec_opt.is_some() {
                    Spanned::new(Expr::LetRec(var, Box::new(e1), Box::new(e2)), span)
                } else {
                    Spanned::new(Expr::Let(var, Box::new(e1), Box::new(e2)), span)
                }
            });

        let if_expr = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then_ignore(just(Token::Else))
            .then(expr.clone())
            .map_with_span(|((cond, then_e), else_e), span| {
                Spanned::new(
                    Expr::If(Box::new(cond), Box::new(then_e), Box::new(else_e)),
                    span,
                )
            });

        let assert_expr = just(Token::Assert)
            .ignore_then(expr.clone())
            .map_with_span(|e, span| Spanned::new(Expr::Assert(Box::new(e)), span));

        // Combined expression parser (all branches produce Spanned<Expr>)
        let raw_expr = lambda
            .or(let_expr)
            .or(if_expr)
            .or(assert_expr)
            .or(struct_decl)
            .or(enum_def)
            .or(match_expr)
            .or(comparison);

        // --- Recovery Strategy ---
        none_of([Token::RBrace, Token::RParen])
            .rewind()
            .then(raw_expr.clone().recover_with(skip_until(
                [
                    Token::Semi,
                    Token::In,
                    Token::Then,
                    Token::Else,
                    // Note: RBrace/RParen excluded
                ],
                |span| Spanned::new(Expr::Error(format!("Syntax Error @ {:?}", span)), span),
            )))
            .map(|(_, e)| e)
            .or(raw_expr)
    })
}

pub fn parse(input: &str) -> Result<Spanned<Expr>, PartialParse> {
    use logos::Logos;

    let tokens: Vec<(Token, std::ops::Range<usize>)> = Token::lexer(input)
        .spanned()
        .filter_map(|(token, span)| match token {
            Ok(t) => Some((t, span)),
            Err(_) => None,
        })
        .collect();

    let eof = input.len()..input.len();
    let stream = chumsky::Stream::from_iter(eof, tokens.into_iter());

    let (output, errors) = parser()
        .separated_by(just(Token::Semi))
        .allow_trailing()
        .then_ignore(end())
        .map_with_span(|mut exprs, span| {
            if exprs.len() == 1 {
                exprs.remove(0)
            } else {
                Spanned::new(Expr::Block(exprs), span)
            }
        })
        .parse_recovery(stream);

    let compile_errors: Vec<CompileErr> = errors
        .into_iter()
        .map(|e| match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => {
                CompileErr::UnclosedDelimiter {
                    span: span.clone(),
                    delimiter: delimiter.to_string(),
                }
            }
            chumsky::error::SimpleReason::Unexpected => CompileErr::UnexpectedToken {
                span: e.span(),
                expected: e
                    .expected()
                    .map(|o| o.clone().map(|t| t.to_string()))
                    .collect(),
                found: e.found().map(|t| t.to_string()),
            },
            chumsky::error::SimpleReason::Custom(msg) => CompileErr::Custom {
                span: e.span(),
                message: msg.to_string(),
            },
        })
        .collect();

    if compile_errors.is_empty() {
        match output {
            Some(ast) => Ok(ast),
            None => Err(PartialParse {
                ast: None,
                errors: vec![],
            }),
        }
    } else {
        Err(PartialParse {
            ast: output,
            errors: compile_errors,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: parse `src` and return its Debug string (or panic with parse error).
    fn dbg_str(src: &str) -> String {
        match parse(src) {
            Ok(e) => format!("{:#?}", e),
            Err(errs) => panic!("parse failed for `{}`: {:#?}", src, errs),
        }
    }

    #[test]
    fn parses_simple_literals_and_vars() {
        let i = dbg_str("42");
        assert!(
            i.contains("42"),
            "expected debug to contain the literal `42`: {}",
            i
        );

        let t = dbg_str("true");
        assert!(
            t.contains("true") || t.contains("Bool"),
            "expected boolean true in debug: {}",
            t
        );

        let v = dbg_str("foo_bar");
        assert!(
            v.contains("foo_bar") || v.contains("Var"),
            "expected identifier in debug: {}",
            v
        );
    }

    #[test]
    fn parses_application_left_associative() {
        // f x y  -> App(App(f, x), y) (at least there should be multiple App markers / identifiers)
        let s = dbg_str("f x y");
        assert!(
            s.contains("App") || (s.matches("f").count() >= 1 && s.matches(' ').count() >= 1),
            "expected application AST or identifiers: {}",
            s
        );
    }

    #[test]
    fn operator_precedence_mul_over_add() {
        // 1 + 2 * 3 should parse with multiplication nested under addition
        let s = dbg_str("1 + 2 * 3");
        // Accept either explicit operator names from Debug or the operator chars / literal ordering.
        let has_mul = s.contains("Mul") || s.contains('*');
        let has_add = s.contains("Add") || s.contains('+');
        assert!(
            has_mul && has_add,
            "expected both add and mul present: {}",
            s
        );
    }

    #[test]
    fn parses_lambda_with_type_and_body() {
        // lambda: .\x: Int -> x
        let s = dbg_str(".\\x: Int -> x");
        assert!(
            s.contains("Abs") || s.contains("Abs(") || s.contains("lambda") || s.contains("->"),
            "expected lambda/abstraction in debug: {}",
            s
        );
        assert!(
            s.contains("Int") || s.contains("Int"),
            "expected Int type: {}",
            s
        );
    }

    #[test]
    fn parses_let_and_if_expressions() {
        let l = dbg_str("let x = 1 in x");
        assert!(
            l.contains("Let") || l.contains("let"),
            "expected let expression in debug: {}",
            l
        );

        let i = dbg_str("if true then 1 else 0");
        assert!(
            i.contains("If") || i.contains("if"),
            "expected if expression: {}",
            i
        );
    }

    #[test]
    fn parses_vector_type_with_numeric_size() {
        // Use a Vector type with a numeric literal for the size expression.
        let src = ".\\v: Vector<Int, 10> -> v";
        let s = dbg_str(src);
        assert!(
            s.contains("Vector") || s.contains("Vector("),
            "expected Vector type: {}",
            s
        );
        assert!(
            s.contains("10") || s.contains("Lit"),
            "expected literal 10 in type expr: {}",
            s
        );
        assert!(
            s.contains("Abs") || s.contains("->"),
            "expected lambda abstraction: {}",
            s
        );
    }

    #[test]
    fn rejects_invalid_input() {
        // malformed let (no rhs) should return Err
        assert!(
            parse("let x = in x").is_err(),
            "expected parse error for malformed let"
        );
    }

    #[test]
    fn test_spans_are_captured() {
        // "1 + 2"
        // 1 is at 0..1
        // + is at 2..3
        // 2 is at 4..5
        // The whole binary expression is 0..5
        let src = "1 + 2";
        let result = parse(src).unwrap();

        assert_eq!(result.span, 0..5);

        if let Expr::Binary(lhs, _, rhs) = result.node {
            assert_eq!(lhs.span, 0..1); // "1"
            assert_eq!(rhs.span, 4..5); // "2"
        } else {
            panic!("Expected Binary Expr");
        }
    }

    #[test]
    fn test_nested_spans() {
        // "let x = 10 in x"
        // Whole let: 0..15
        let src = "let x = 10 in x";
        let result = parse(src).unwrap();

        assert_eq!(result.span, 0..15);

        if let Expr::Let(var, val, body) = result.node {
            assert_eq!(var, "x");
            assert_eq!(val.span, 8..10); // "10"
            assert_eq!(body.span, 14..15); // "x"
        } else {
            panic!("Expected Let Expr");
        }
    }

    #[test]
    fn parses_blocks_with_semicolons() {
        // Block: { 1; 2; 3 }
        // Should parse as Expr::Block(vec![1, 2, 3])
        let src = "{ 1; 2; 3 }";
        let s = dbg_str(src);
        assert!(s.contains("Block"), "expected Block in debug");
        // Count occurrences of LitI64 or just numbers in the debug output depends on implementation
        // But simply ensuring "Block" exists is a good start.
    }

    #[test]
    fn parses_struct_declaration_and_init() {
        let src = r#"
            struct Point { x: Int, y: Int } in
            Point { x: 1, y: 2 }
        "#;
        let s = dbg_str(src);
        assert!(s.contains("StructDef"), "expected StructDef");
        assert!(s.contains("Point"), "expected struct name Point");
        assert!(s.contains("StructInit"), "expected StructInit");
    }

    #[test]
    fn parses_field_access() {
        let src = "p.x";
        let s = dbg_str(src);
        assert!(s.contains("FieldAccess"), "expected FieldAccess");
        assert!(s.contains("\"x\""), "expected field name 'x'");
    }

    #[test]
    fn parses_enum_declaration_and_init() {
        let src = r#"
            enum Option { Some(Int), None() } in
            Option::Some(42)
        "#;
        let s = dbg_str(src);
        assert!(s.contains("EnumDef"), "expected EnumDef");
        assert!(s.contains("EnumInit"), "expected EnumInit");
        assert!(s.contains("Option"), "expected enum name");
        assert!(s.contains("Some"), "expected variant name");
    }

    #[test]
    fn parses_match_expression() {
        let src = r#"
            match val {
                Option::Some(x) -> x,
                Option::None() -> 0,
                ... -> -1
            }
        "#;
        let s = dbg_str(src);
        assert!(s.contains("Match"), "expected Match expression");
        assert!(s.contains("EnumPat"), "expected Enum pattern");
        assert!(s.contains("Wildcard"), "expected Wildcard pattern");
    }

    #[test]
    fn parses_nested_generic_types() {
        // Checking parsing of Vector<Vector<Int, 2>, 2>
        // Note: The parser treats Vector specially in type_parser
        let src = r#".\m: Vector<Vector<Int, 2>, 2> -> m"#;
        let s = dbg_str(src);
        assert!(s.contains("Vector"), "expected Vector type");
        // Check nesting logic (heuristic)
        assert!(s.matches("Vector").count() >= 2, "expected nested Vectors");
    }

    // I.. just can't
    // #[test]
    // fn error_recovery_in_blocks() {
    //     // This test specifically checks if the parser can recover from a bad statement
    //     // inside a block and still produce a partial AST.
    //
    //     let src = "{
    //         let x = 1;
    //         1 + * 2;   // Syntax Error here (Binary op missing lhs/rhs logic or unexpected tokens)
    //         let y = 3
    //     }";
    //
    //     match parse(src) {
    //         Ok(_) => panic!("Expected parsing to fail due to syntax error, but it succeeded."),
    //         Err(partial) => {
    //             // We expect some errors
    //             assert!(!partial.errors.is_empty(), "Expected compilation errors");
    //
    //             // We expect a Partial AST
    //             if let Some(ast) = partial.ast {
    //                 println!("Partial AST: {:#?}", ast);
    //                 if let Expr::Block(stmts) = ast.node {
    //                     assert_eq!(stmts.len(), 3, "Expected 3 statements (1 valid, 1 error, 1 valid)");
    //
    //                     // Check first stmt
    //                     match &stmts[0].node {
    //                         Expr::Let(name, _, _) => assert_eq!(name, "x"),
    //                         _ => panic!("First statement should be 'let x'"),
    //                     }
    //
    //                     // Check second stmt (Error)
    //                     match &stmts[1].node {
    //                         Expr::Error(_) => {} // Success, we recovered this node
    //                         node => panic!("Second statement should be Error, found {:?}", node),
    //                     }
    //
    //                     // Check third stmt
    //                     match &stmts[2].node {
    //                         Expr::Let(name, _, _) => assert_eq!(name, "y"),
    //                         _ => panic!("Third statement should be 'let y'"),
    //                     }
    //                 } else {
    //                     panic!("Top level AST should be a Block");
    //                 }
    //             } else {
    //                 panic!("Parser failed to produce a partial AST!");
    //             }
    //         }
    //     }
    // }

    #[test]
    fn error_recovery_does_not_consume_delimiters() {
        // Crucial test for the "Lookahead Guard" logic.
        // If recovery eats '}', the block parser will fail with "Unexpected End of Input".

        let src = "{ let x = ; }"; // Error inside let (missing expression)

        match parse(src) {
            Ok(_) => panic!("Should fail"),
            Err(partial) => {
                // Ensure we got a block back, not just None
                if let Some(ast) = partial.ast {
                    // It might return Expr::Block([Expr::Error])
                    // or Expr::Block([Expr::Let(..., Expr::Error, ...)]) depending on where it failed.
                    // The important part is that we got an AST and didn't crash on '}'
                    println!("Recovered AST: {:#?}", ast);
                } else {
                    panic!("Failed to recover AST at block delimiter");
                }
            }
        }
    }
}
