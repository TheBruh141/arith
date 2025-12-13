use arith::syntax::ast::{BinaryOp, Expr, Type};
use arith::syntax::parser::parse;

#[test]
fn test_simple_arithmetic() {
    let src = "1 + 2 * 3";
    let expr = parse(src).unwrap();
    println!("{:?}", expr);
    // 1 + (2 * 3)
    match expr {
        Expr::Binary(lhs, BinaryOp::Add, rhs) => {
            assert!(matches!(*lhs, Expr::Int(1)));
            match *rhs {
                Expr::Binary(rl, BinaryOp::Mul, rr) => {
                    assert!(matches!(*rl, Expr::Int(2)));
                    assert!(matches!(*rr, Expr::Int(3)));
                }
                _ => panic!("Expected multiplication on rhs"),
            }
        }
        _ => panic!("Expected addition at top level"),
    }
}

#[test]
fn test_lambda_parse() {
    let src = ".\\x:Int -> x + 1";
    let expr = parse(src).unwrap();
    if let Expr::Abs(var, ty, body) = expr {
        assert_eq!(var, "x");
        assert_eq!(ty, Type::Int);
        // Body should be Binary(Var(x) + Int(1))
    } else {
        panic!("Failed to parse lambda");
    }
}

#[test]
fn test_vector_type() {
    // We don't have a way to constructing a value of type Vector yet in the syntax (maybe just variables)
    // But we can check if it parses inside a lambda type
    let src = ".\\v:Vector<Int, 4> -> v";
    let expr = parse(src).unwrap();
    // Check type parsing
}
