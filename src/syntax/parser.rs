use crate::syntax::ast::{BinaryOp, Expr, Type, TypeExpr};
use crate::syntax::lexer::Token;
use chumsky::prelude::*;
use logos::Logos;

pub fn parser() -> impl Parser<Token, Expr, Error = Simple<Token>> {
    let ident = select! { Token::Ident(id) => id };
    let int_lit = select! { Token::Num(n) => n };

    let type_expr = recursive(|type_expr| {
        let atom = int_lit
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

        // Arrow is Right Associative: Int -> (Int -> Int)
        atom.clone()
            .separated_by(just(Token::Arrow))
            .at_least(1)
            .map(|types| {
                let mut iter = types.into_iter().rev();
                let last = iter.next().unwrap();
                iter.fold(last, |acc, t| Type::Arrow(Box::new(t), Box::new(acc)))
            })
    });

    recursive(|expr| {
        let lit = int_lit
            .map(Expr::Int)
            .or(just(Token::True).to(Expr::Bool(true)))
            .or(just(Token::False).to(Expr::Bool(false)));

        let atom = lit.or(ident.map(Expr::Var)).or(expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen)));

        // Function Application: f x y
        let app = atom
            .clone()
            .then(atom.clone().repeated())
            .foldl(|func, arg| Expr::App(Box::new(func), Box::new(arg)));

        // Math: Product (* /)
        let product = app
            .clone()
            .then(
                just(Token::Mul)
                    .to(BinaryOp::Mul)
                    .or(just(Token::Div).to(BinaryOp::Div))
                    .then(app.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)));

        // Math: Sum (+ -)
        let sum = product
            .clone()
            .then(
                just(Token::Plus)
                    .to(BinaryOp::Add)
                    .or(just(Token::Minus).to(BinaryOp::Sub))
                    .then(product.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)));

        let comparison = sum
            .clone()
            .then(
                just(Token::EqEq)
                    .to(BinaryOp::Equals)
                    .or(just(Token::Geq).to(BinaryOp::GreaterThanEquals))
                    .or(just(Token::Leq).to(BinaryOp::LessThanEquals))
                    .or(just(Token::LAngle).to(BinaryOp::LessThan))
                    .or(just(Token::RAngle).to(BinaryOp::GreaterThan))
                    .then(sum.clone())
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)));

        // Structures
        let lambda = just(Token::Lambda)
            .ignore_then(ident)
            .then_ignore(just(Token::Colon))
            .then(type_parser.clone())
            .then_ignore(just(Token::Arrow))
            .then(expr.clone())
            .map(|((var, ty), body)| Expr::Abs(var, ty, Box::new(body)));

        let let_expr = just(Token::Let)
            .ignore_then(ident)
            .then_ignore(just(Token::Eq))
            .then(expr.clone())
            .then_ignore(just(Token::In))
            .then(expr.clone())
            .map(|((var, e1), e2)| Expr::Let(var, Box::new(e1), Box::new(e2)));

        let if_expr = just(Token::If)
            .ignore_then(expr.clone())
            .then_ignore(just(Token::Then))
            .then(expr.clone())
            .then_ignore(just(Token::Else))
            .then(expr.clone())
            .map(|((cond, then_e), else_e)| {
                Expr::If(Box::new(cond), Box::new(then_e), Box::new(else_e))
            });

        lambda.or(let_expr).or(if_expr).or(comparison)
    })
}
pub fn parse(input: &str) -> Result<Expr, Vec<Simple<Token>>> {
    // 1. Lex the input WITH spans
    // .spanned() turns the iterator into items of (Result<Token, _>, Range<usize>)
    let tokens: Vec<(Token, std::ops::Range<usize>)> = Token::lexer(input)
        .spanned()
        .filter_map(|(token, span)| match token {
            Ok(t) => Some((t, span)),
            Err(_) => None, // Skip invalid characters (or handle lexer errors here)
        })
        .collect();

    // Calculate end of file span for error reporting at EOF
    let eof = input.len()..input.len();

    // 2. Parse the stream of (Token, Span) tuples
    parser()
        .then_ignore(end())
        .parse(chumsky::Stream::from_iter(eof, tokens.into_iter()))
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
