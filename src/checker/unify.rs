use crate::syntax::ast::{BinaryOp, Type, TypeExpr};
use log::debug;

/// Unify two types.
/// For now, this performs strict structural unification.
/// In the future, it should handle type variables (for inference) and arithmetic reduction.
pub fn unify(t1: &Type, t2: &Type) -> Result<(), (Type, Type)> {
    match (t1, t2) {
        (Type::Int, Type::Int) => Ok(()),
        (Type::Bool, Type::Bool) => Ok(()),
        (Type::Arrow(a1, r1), Type::Arrow(a2, r2)) => {
            unify(a1, a2)?;
            unify(r1, r2)
        }
        (Type::Vector(v1, dim1), Type::Vector(v2, dim2)) => {
            unify(v1, v2)?;
            unify_dim(dim1, dim2).map_err(|_| (t1.clone(), t2.clone()))
        }
        (Type::Struct(n1), Type::Struct(n2)) if n1 == n2 => Ok(()),

        // Primitives
        (Type::I8, Type::I8) => Ok(()),
        (Type::I16, Type::I16) => Ok(()),
        (Type::I32, Type::I32) => Ok(()),
        (Type::I64, Type::I64) => Ok(()),
        (Type::Isize, Type::Isize) => Ok(()),
        (Type::U8, Type::U8) => Ok(()),
        (Type::U16, Type::U16) => Ok(()),
        (Type::U32, Type::U32) => Ok(()),
        (Type::U64, Type::U64) => Ok(()),
        (Type::Usize, Type::Usize) => Ok(()),
        (Type::F16, Type::F16) => Ok(()),
        (Type::F32, Type::F32) => Ok(()),
        (Type::F64, Type::F64) => Ok(()),

        (t1, t2) => Err((t1.clone(), t2.clone())),
    }
}

/// Unify two type-level expressions (dimensions).
fn unify_dim(d1: &TypeExpr, d2: &TypeExpr) -> Result<(), (TypeExpr, TypeExpr)> {
    // Basic solver: Structural equality + simple simplification?
    // For now: Strict equality checking.
    // TODO: Implement actual arithmetic solver (e.g. normalize to polynomial form)

    if d1 == d2 {
        Ok(())
    } else {
        // Try to evaluate constant expressions
        if let (Some(n1), Some(n2)) = (eval_const(d1), eval_const(d2))
            && n1 == n2
        {
            return Ok(());
        }

        Err((d1.clone(), d2.clone()))
    }
}
/// Compile-Time Constant Evaluator
///
/// This function attempts to run math *inside the type system*.
/// Input: A type expression (like `2 + 2` inside `Vector<Int, 2+2>`)
/// Output: The calculated integer, or None if it can't be calculated yet.
fn eval_const(expr: &TypeExpr) -> Option<i64> {
    match expr {
        TypeExpr::Lit(n) => Some(*n),
        TypeExpr::Binary(lhs, op, rhs) => {
            let l = eval_const(lhs)?;
            let r = eval_const(rhs)?;
            match op {
                BinaryOp::Add => Some(l + r),
                BinaryOp::Sub => Some(l - r),
                BinaryOp::Mul => Some(l * r),
                BinaryOp::Div => {
                    if r != 0 {
                        Some(l / r)
                    } else {
                        None
                    }
                }
                _ => {
                    debug!("[unify-eval_const]unhandled eval_const {op}");
                    None
                }
            }
        }
        TypeExpr::Var(_) => None,
    }
}
