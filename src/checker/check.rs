use crate::checker::unify::{Result, TypeError, unify};
use crate::syntax::ast::{BinaryOp, Expr, Type};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Context {
    vars: HashMap<String, Type>,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
}

pub fn check_expr(ctx: &Context, expr: &Expr) -> Result<Type> {
    match expr {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),

        Expr::Var(name) => ctx
            .get(name)
            .cloned()
            .ok_or_else(|| TypeError::UnknownVar(name.clone())),

        Expr::Abs(var, ty, body) => {
            let mut new_ctx = ctx.clone();
            new_ctx.insert(var.clone(), ty.clone());
            let body_ty = check_expr(&new_ctx, body)?;
            Ok(Type::Arrow(Box::new(ty.clone()), Box::new(body_ty)))
        }

        Expr::App(func, arg) => {
            let func_ty = check_expr(ctx, func)?;
            let arg_ty = check_expr(ctx, arg)?;

            match func_ty {
                Type::Arrow(param_ty, return_ty) => {
                    unify(&param_ty, &arg_ty)?;
                    Ok(*return_ty)
                }
                _ => Err(TypeError::Mismatch(
                    Type::Arrow(Box::new(arg_ty.clone()), Box::new(Type::Int)),
                    func_ty,
                )), // Better error needed
            }
        }

        Expr::Binary(lhs, op, rhs) => {
            let l_ty = check_expr(ctx, lhs)?;
            let r_ty = check_expr(ctx, rhs)?;

            // For now, only Int arithmetic
            unify(&l_ty, &Type::Int)?;
            unify(&r_ty, &Type::Int)?;

            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => Ok(Type::Int),
                // If we add comparison ops later, they return Bool
                BinaryOp::Equals
                | BinaryOp::LessThan
                | BinaryOp::GreaterThan
                | BinaryOp::LessThanEquals
                | BinaryOp::GreaterThanEquals => Ok(Type::Bool),
            }
        }

        Expr::Let(var, e1, e2) => {
            let t1 = check_expr(ctx, e1)?;
            let mut new_ctx = ctx.clone();
            new_ctx.insert(var.clone(), t1);
            check_expr(&new_ctx, e2)
        }

        Expr::If(cond, e_then, e_else) => {
            let t_cond = check_expr(ctx, cond)?;
            unify(&t_cond, &Type::Bool)?;

            let t_then = check_expr(ctx, e_then)?;
            let t_else = check_expr(ctx, e_else)?;

            unify(&t_then, &t_else)?;
            Ok(t_then)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::unify::TypeError;
    use crate::syntax::parser::parse;

    macro_rules! check {
        ($input:expr, $expected:expr) => {
            match parse($input) {
                Ok(_) => {
                    assert_eq!(ty($input, Context::new()).unwrap(), $expected)
                }
                Err(err) => panic!("parse failed for `{}`: {:#?}", $input, err),
            }
        };
    }

    fn ctx(vars: &[(&str, Type)]) -> Context {
        let mut c = Context::new();
        for (name, ty) in vars {
            c.insert(name.to_string(), ty.clone());
        }
        c
    }

    fn ty(src: &str, ctx: Context) -> Result<Type> {
        let expr = parse(src).unwrap_or_else(|e| panic!("parse failed for `{src}`: {e:#?}"));
        check_expr(&ctx, &expr)
    }

    // ---------- tests ----------

    #[test]
    fn int_and_bool_literals() {
        check!("1", Type::Int);
        check!("true", Type::Bool);
    }

    #[test]
    fn known_and_unknown_variables() {
        let c = ctx(&[("x", Type::Int)]);
        assert_eq!(check_expr(&c, &Expr::Var("x".into())).unwrap(), Type::Int);

        let err = check_expr(&Context::new(), &Expr::Var("nope".into())).unwrap_err();
        match err {
            TypeError::UnknownVar(v) => assert_eq!(v, "nope"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn lambda_typing() {
        let e = Expr::Abs("x".into(), Type::Int, Box::new(Expr::Var("x".into())));
        let ty = check_expr(&Context::new(), &e).unwrap();
        assert_eq!(ty, Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)));
    }

    #[test]
    fn app_correct() {
        let lam = Expr::Abs("x".into(), Type::Int, Box::new(Expr::Var("x".into())));
        let app = Expr::App(Box::new(lam), Box::new(Expr::Int(1)));

        let ty = check_expr(&Context::new(), &app).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn arithmetic_on_ints() {
        check!("1 + 2", Type::Int);
    }

    #[test]
    fn arithmetic_type_error() {
        let e = Expr::Binary(
            Box::new(Expr::Int(1)),
            BinaryOp::Add,
            Box::new(Expr::Bool(true)),
        );
        let err = check_expr(&Context::new(), &e).unwrap_err();
        matches!(err, TypeError::Mismatch(_, _));
    }

    #[test]
    fn let_binding() {
        let e = Expr::Let(
            "x".into(),
            Box::new(Expr::Int(1)),
            Box::new(Expr::Binary(
                Box::new(Expr::Var("x".into())),
                BinaryOp::Add,
                Box::new(Expr::Int(2)),
            )),
        );
        assert_eq!(check_expr(&Context::new(), &e).unwrap(), Type::Int);
    }

    #[test]
    fn if_expression_ok() {
        check!("if true then 1 else 2", Type::Int);
    }

    #[test]
    fn ints_bools_vars() {
        check!("1", Type::Int);
        check!("true", Type::Bool);

        let c = ctx(&[("x", Type::Int)]);
        assert_eq!(ty("x", c).unwrap(), Type::Int);

        let err = ty("y", Context::new()).unwrap_err();
        matches!(err, TypeError::UnknownVar(_));
    }

    #[test]
    fn lambda_simple() {
        check!(
            ".\\x: Int -> x",
            Type::Arrow(Box::new(Type::Int), Box::new(Type::Int))
        );
    }

    #[test]
    fn lambda_nested() {
        let out = ty(".\\x: Int -> .\\y: Int -> x", Context::new()).unwrap();
        match out {
            Type::Arrow(a, b) => {
                assert_eq!(*a, Type::Int);
                match *b {
                    Type::Arrow(inner_a, inner_b) => {
                        assert_eq!(*inner_a, Type::Int);
                        assert_eq!(*inner_b, Type::Int);
                    }
                    other => panic!("expected nested arrow, got {other:?}"),
                }
            }
            other => panic!("expected arrow, got {other:?}"),
        }
    }

    #[test]
    fn app_ok() {
        check!("(.\\x: Int -> x) 10", Type::Int);
    }

    #[test]
    fn app_type_mismatch() {
        let err = ty("(.\\x: Int -> x) true", Context::new()).unwrap_err();
        matches!(err, TypeError::Mismatch(_, _));
    }

    #[test]
    fn arithmetic_ok() {
        check!("1 + 2 * 3", Type::Int);
    }

    #[test]
    fn arithmetic_err() {
        let err = ty("1 + true", Context::new()).unwrap_err();
        matches!(err, TypeError::Mismatch(_, _));
    }

    #[test]
    fn let_ok() {
        check!("let x = 1 in x + 2", Type::Int);
    }

    #[test]
    fn if_ok() {
        check!("if true then 1 else 2", Type::Int);
    }

    #[test]
    fn if_condition_must_be_bool() {
        let err = ty("if 1 then 2 else 3", Context::new()).unwrap_err();
        matches!(err, TypeError::Mismatch(_, _));
    }

    #[test]
    fn if_branches_must_match() {
        let err = ty("if true then 1 else false", Context::new()).unwrap_err();
        matches!(err, TypeError::Mismatch(_, _));
    }

    #[test]
    fn arrow_type_and_application_chain() {
        check!("(.\\x: Int -> .\\y: Int -> x) 10 20", Type::Int);
    }
}
