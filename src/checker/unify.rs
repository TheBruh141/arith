use crate::syntax::ast::{BinaryOp, Type, TypeExpr};

#[derive(Debug)]
pub enum TypeError {
    Mismatch(Type, Type),
    DimensionMismatch(TypeExpr, TypeExpr),
    UnknownVar(String),
}

pub type Result<T> = std::result::Result<T, TypeError>;

/// Unify two types.
/// For now, this performs strict structural unification.
/// In the future, it should handle type variables (for inference) and arithmetic reduction.
pub fn unify(t1: &Type, t2: &Type) -> Result<()> {
    match (t1, t2) {
        (Type::Int, Type::Int) => Ok(()),
        (Type::Bool, Type::Bool) => Ok(()),
        (Type::Arrow(a1, r1), Type::Arrow(a2, r2)) => {
            unify(a1, a2)?;
            unify(r1, r2)
        }
        (Type::Vector(v1, dim1), Type::Vector(v2, dim2)) => {
            unify(v1, v2)?;
            unify_dim(dim1, dim2)
        }
        (t1, t2) => Err(TypeError::Mismatch(t1.clone(), t2.clone())),
    }
}

/// Unify two type-level expressions (dimensions).
fn unify_dim(d1: &TypeExpr, d2: &TypeExpr) -> Result<()> {
    // Basic solver: Structural equality + simple simplification?
    // For now: Strict equality checking.
    // TODO: Implement actual arithmetic solver (e.g. normalize to polynomial form)

    if d1 == d2 {
        Ok(())
    } else {
        // Try to evaluate constant expressions
        if let (Some(n1), Some(n2)) = (eval_const(d1), eval_const(d2)) {
            if n1 == n2 {
                return Ok(());
            }
        }

        Err(TypeError::DimensionMismatch(d1.clone(), d2.clone()))
    }
}

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
            }
        }
        TypeExpr::Var(_) => None,
    }
}
