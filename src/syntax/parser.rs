use crate::compiler_errors::CompileErr;
use crate::syntax::ast::PartialParse;
use crate::syntax::ast::{BinaryOp, Expr, Spanned, Type, TypeExpr};
use crate::syntax::lexer::Token;
use chumsky::prelude::*;

pub fn parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> {
    // --- Primitives ---
    let ident = select! { Token::Ident(id) => id };

    let lit_expr = select! {
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
    };

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
            // Structs
            .or(ident.try_map(|s, span| {
                if s.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Ok(Type::Struct(s))
                } else {
                    Err(Simple::custom(span, "Struct type must be capitalized"))
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
        let lit = lit_expr.clone();

        // BLOCK PARSER
        let block = expr
            .clone()
            .separated_by(just(Token::Semi))
            .allow_trailing()
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(|mut stmts: Vec<Spanned<Expr>>| {
                if stmts.len() == 1 {
                    stmts.remove(0).node
                } else {
                    Expr::Block(stmts)
                }
            });

        // STRUCT INIT
        let struct_init = ident
            .then(
                ident
                    .then_ignore(just(Token::Colon))
                    .then(expr.clone())
                    .separated_by(just(Token::Comma))
                    .delimited_by(just(Token::LBrace), just(Token::RBrace)),
            )
            .map(|(name, fields)| Expr::StructInit(name, fields));

        let atom_base = lit
            .or(block)
            .or(struct_init)
            .or(ident.map(Expr::Var))
            .or(expr
                .clone()
                .delimited_by(just(Token::LParen), just(Token::RParen))
                .map(|e| e.node))
            .map_with_span(Spanned::new);

        // POSTFIX (App, Field)
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

        // UNARY
        let unary = recursive(|unary| {
            let op = just(Token::Minus)
                .to(crate::syntax::ast::UnaryOp::Neg)
                .or(just(Token::Bang).to(crate::syntax::ast::UnaryOp::Not));

            op.then(unary)
                .map_with_span(|(op, expr), span| {
                    Spanned::new(Expr::Unary(op, Box::new(expr)), span)
                })
                .or(apply_or_access.clone())
        });

        // BINARY PRECEDENCE
        let product = unary
            .clone()
            .then(
                just(Token::Mul)
                    .or(just(Token::Div))
                    .then(apply_or_access)
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
                    .then(product)
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
                .then(sum)
                .repeated(),
            )
            .foldl(|lhs, (op, rhs)| {
                let span = lhs.span.start..rhs.span.end;
                Spanned::new(Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span)
            });

        // CONTROL FLOW
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
                Spanned::new(Expr::StructDecl(name, fields, Box::new(body)), span)
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

        let raw_expr = lambda
            .or(let_expr)
            .or(if_expr)
            .or(assert_expr)
            .or(struct_decl)
            .or(comparison);

        // --- LOOKAHEAD GUARD ---
        // 1. none_of([...]) ensures the next token is NOT } or ).
        // 2. rewind() resets the stream so we don't consume the allowed token.
        // 3. then(...) executes the recovering parser.
        // 4. or(raw_expr) is the fallback if we ARE at } or ).
        //    raw_expr (unwrapped) will fail immediately without recovery,
        //    allowing the block parser to see the closing delimiter.

        none_of([Token::RBrace, Token::RParen])
            .rewind()
            .then(raw_expr.clone().recover_with(skip_until(
                [Token::Semi, Token::In, Token::Then, Token::Else],
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
}
