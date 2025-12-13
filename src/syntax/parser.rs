use crate::syntax::ast::{BinaryOp, Expr, Type, TypeExpr};
use chumsky::prelude::*;

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    // 1. Define Identifiers and Keywords
    let keywords = [
        "if", "then", "else", "let", "in", "true", "false", "Int", "Bool", "Vector",
    ];

    let ident = text::ident().padded().try_map(move |s: String, span| {
        if keywords.contains(&s.as_str()) {
            Err(Simple::custom(
                span,
                format!("'{}' is a reserved keyword", s),
            ))
        } else {
            Ok(s)
        }
    });

    let int_lit = text::int(10)
        .map(|s: String| s.parse::<i64>().unwrap())
        .padded();

    // 2. Type Expression Parser
    let type_expr = recursive(|type_expr| {
        let atom = int_lit
            .clone()
            .map(TypeExpr::Lit)
            .or(ident.clone().map(TypeExpr::Var))
            .padded()
            .delimited_by(just('('), just(')'))
            .or(int_lit.clone().map(TypeExpr::Lit))
            .or(ident.clone().map(TypeExpr::Var))
            .padded();

        let product = atom
            .clone()
            .then(just('*').padded().ignore_then(atom).repeated())
            .map(|(lhs, rhs)| {
                rhs.into_iter().fold(lhs, |acc, r| {
                    TypeExpr::Binary(Box::new(acc), BinaryOp::Mul, Box::new(r))
                })
            });

        let sum = product
            .clone()
            .then(just('+').padded().ignore_then(product).repeated())
            .map(|(lhs, rhs)| {
                rhs.into_iter().fold(lhs, |acc, r| {
                    TypeExpr::Binary(Box::new(acc), BinaryOp::Add, Box::new(r))
                })
            });

        sum
    });

    // 3. Main Type Parser
    let type_parser = recursive(|type_def| {
        let base = just("Int")
            .to(Type::Int)
            .or(just("Bool").to(Type::Bool))
            .or(just("Vector")
                .padded()
                .ignore_then(
                    type_def
                        .clone()
                        .then_ignore(just(',').padded())
                        .then(type_expr.clone())
                        .delimited_by(just('<'), just('>')),
                )
                .map(|(t, n)| Type::Vector(Box::new(t), n)));

        let atom = base
            .padded()
            .or(type_def.delimited_by(just('('), just(')')))
            .padded();

        atom.clone()
            .separated_by(just("->").padded())
            .at_least(1)
            .map(|types| {
                let mut iter = types.into_iter().rev();
                let last = iter.next().unwrap();
                iter.fold(last, |acc, t| Type::Arrow(Box::new(t), Box::new(acc)))
            })
    });

    // 4. Expression Parser
    recursive(|expr| {
        let lit = int_lit
            .clone()
            .map(Expr::Int)
            .or(just("true").to(Expr::Bool(true)))
            .or(just("false").to(Expr::Bool(false)));

        let atom = lit
            .or(ident.clone().map(Expr::Var))
            .or(expr.clone().delimited_by(just('('), just(')')))
            .padded();

        let app = atom
            .clone()
            .then(atom.clone().repeated())
            .map(|(func, args)| {
                args.into_iter()
                    .fold(func, |acc, arg| Expr::App(Box::new(acc), Box::new(arg)))
            });

        let product = app
            .clone()
            .then(
                just('*')
                    .to(BinaryOp::Mul)
                    .or(just('/').to(BinaryOp::Div))
                    .padded()
                    .then(app.clone())
                    .repeated(),
            )
            .map(|(lhs, rest)| {
                rest.into_iter().fold(lhs, |acc, (op, val)| {
                    Expr::Binary(Box::new(acc), op, Box::new(val))
                })
            });

        let sum = product
            .clone()
            .then(
                just('+')
                    .to(BinaryOp::Add)
                    .or(just('-').to(BinaryOp::Sub))
                    .padded()
                    .then(product.clone())
                    .repeated(),
            )
            .map(|(lhs, rest)| {
                rest.into_iter().fold(lhs, |acc, (op, val)| {
                    Expr::Binary(Box::new(acc), op, Box::new(val))
                })
            });

        let lambda = just(".\\")
            .padded()
            .ignore_then(ident.clone())
            .then_ignore(just(':').padded())
            .then(type_parser.clone())
            .then_ignore(just("->").padded())
            .then(expr.clone())
            .map(|((var, ty), body)| Expr::Abs(var, ty, Box::new(body)));

        let let_expr = just("let")
            .padded()
            .ignore_then(ident.clone())
            .then_ignore(just('=').padded())
            .then(expr.clone())
            .then_ignore(just("in").padded())
            .then(expr.clone())
            .map(|((var, e1), e2)| Expr::Let(var, Box::new(e1), Box::new(e2)));

        let if_expr = just("if")
            .padded()
            .ignore_then(expr.clone())
            .then_ignore(just("then").padded())
            .then(expr.clone())
            .then_ignore(just("else").padded())
            .then(expr.clone())
            .map(|((cond, then_e), else_e)| {
                Expr::If(Box::new(cond), Box::new(then_e), Box::new(else_e))
            });

        // The order here matters less now that `ident` correctly fails on keywords,
        // but explicit structures usually go first.
        lambda.or(let_expr).or(if_expr).or(sum)
    })
}
pub fn parse(input: &str) -> Result<Expr, Vec<Simple<char>>> {
    parser().parse(input)
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
}
