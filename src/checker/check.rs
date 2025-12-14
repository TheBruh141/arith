use crate::checker::unify::unify;
use crate::compiler_errors::{CompileErr, CompileResult};
use crate::syntax::ast::{BinaryOp, Expr, Spanned, Type};
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

pub fn check_expr(ctx: &Context, expr: &Spanned<Expr>) -> CompileResult<Type> {
    match &expr.node {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),

        Expr::Var(name) => ctx.get(name).cloned().ok_or_else(|| CompileErr::UnknownVar {
            span: expr.span.clone(),
            name: name.clone(),
        }),

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
                    unify(&arg_ty, &param_ty).map_err(|(found, expected)| {
                        CompileErr::TypeMismatch {
                            span: arg.span.clone(),
                            expected,
                            found,
                        }
                    })?;
                    Ok(*return_ty)
                }
                other_ty => Err(CompileErr::TypeMismatch {
                    span: func.span.clone(),
                    expected: Type::Arrow(Box::new(arg_ty), Box::new(Type::Int)), // Placeholder
                    found: other_ty,
                }),
            }
        }

        Expr::Binary(lhs, op, rhs) => {
            let l_ty = check_expr(ctx, lhs)?;
            let r_ty = check_expr(ctx, rhs)?;

            let (expected_ty, out_ty) = match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    (Type::Int, Type::Int)
                }
                BinaryOp::Equals
                | BinaryOp::LessThan
                | BinaryOp::GreaterThan
                | BinaryOp::LessThanEquals
                | BinaryOp::GreaterThanEquals => (Type::Int, Type::Bool), // For now, only compare ints
            };

            unify(&l_ty, &expected_ty).map_err(|(found, expected)| {
                CompileErr::TypeMismatch {
                    span: lhs.span.clone(),
                    expected,
                    found,
                }
            })?;
            unify(&r_ty, &expected_ty).map_err(|(found, expected)| {
                CompileErr::TypeMismatch {
                    span: rhs.span.clone(),
                    expected,
                    found,
                }
            })?;

            Ok(out_ty)
        }

        Expr::Let(var, e1, e2) => {
            let t1 = check_expr(ctx, e1)?;
            let mut new_ctx = ctx.clone();
            new_ctx.insert(var.clone(), t1);
            check_expr(&new_ctx, e2)
        }

        Expr::If(cond, e_then, e_else) => {
            let t_cond = check_expr(ctx, cond)?;
            unify(&t_cond, &Type::Bool).map_err(|(found, expected)| {
                CompileErr::TypeMismatch {
                    span: cond.span.clone(),
                    expected,
                    found,
                }
            })?;

            let t_then = check_expr(ctx, e_then)?;
            let t_else = check_expr(ctx, e_else)?;

            unify(&t_else, &t_then).map_err(|(found, expected)| CompileErr::TypeMismatch {
                span: e_else.span.clone(),
                expected,
                found,
            })?;
            Ok(t_then)
        }
    }
}
