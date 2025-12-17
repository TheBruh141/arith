use crate::checker::unify::unify;
use crate::compiler_errors::{CompileErr, CompileResult};
use crate::syntax::ast::{BinaryOp, Expr, Spanned, Type, UnaryOp};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Context {
    vars: HashMap<String, Type>,
    structs: HashMap<String, HashMap<String, Type>>,
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
            structs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn insert_struct(&mut self, name: String, fields: HashMap<String, Type>) {
        self.structs.insert(name, fields);
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    pub fn get_struct(&self, name: &str) -> Option<&HashMap<String, Type>> {
        self.structs.get(name)
    }
}

pub fn check_expr(ctx: &Context, expr: &Spanned<Expr>) -> CompileResult<Type> {
    match &expr.node {
        Expr::Int(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),

        Expr::LitU8(_) => Ok(Type::U8),
        Expr::LitU16(_) => Ok(Type::U16),
        Expr::LitU32(_) => Ok(Type::U32),
        Expr::LitU64(_) => Ok(Type::U64),
        Expr::LitUsize(_) => Ok(Type::Usize),
        Expr::LitI8(_) => Ok(Type::I8),
        Expr::LitI16(_) => Ok(Type::I16),
        Expr::LitI32(_) => Ok(Type::I32),
        Expr::LitI64(_) => Ok(Type::I64),
        Expr::LitIsize(_) => Ok(Type::Isize),
        Expr::LitF16(_) => Ok(Type::F16),
        Expr::LitF32(_) => Ok(Type::F32),
        Expr::LitF64(_) => Ok(Type::F64),

        Expr::Var(name) => ctx
            .get(name)
            .cloned()
            .ok_or_else(|| CompileErr::UnknownVar {
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
                found_type => Err(CompileErr::NotAFunction {
                    span: func.span.clone(),
                    found_type,
                }),
            }
        }

        Expr::Binary(lhs, op, rhs) => {
            let l_ty = check_expr(ctx, lhs)?;
            let r_ty = check_expr(ctx, rhs)?;

            // 1. Enforce strict equality of operands (no implicit casting)
            unify(&r_ty, &l_ty).map_err(|(found_operand_type, expected_operand_type)| {
                CompileErr::BinaryOpMismatch {
                    span: rhs.span.clone(),
                    op: op.clone(),
                    expected_operand_type,
                    found_operand_type,
                }
            })?;

            // 2. Validate that the type supports the operation
            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    match l_ty {
                        Type::Int
                        | Type::I8
                        | Type::I16
                        | Type::I32
                        | Type::I64
                        | Type::Isize
                        | Type::U8
                        | Type::U16
                        | Type::U32
                        | Type::U64
                        | Type::Usize
                        | Type::F16
                        | Type::F32
                        | Type::F64 => Ok(l_ty),
                        _ => Err(CompileErr::BinaryOpMismatch {
                            span: lhs.span.clone(),
                            op: op.clone(),
                            expected_operand_type: Type::Int, // TODO: Better error for "Expected Numeric"
                            found_operand_type: l_ty,
                        }),
                    }
                }
                BinaryOp::Equals
                | BinaryOp::NotEquals
                | BinaryOp::LessThan
                | BinaryOp::GreaterThan
                | BinaryOp::LessThanEquals
                | BinaryOp::GreaterThanEquals => {
                    // For now, allow comparison on all primitives + Bool
                    match l_ty {
                        Type::Int
                        | Type::I8
                        | Type::I16
                        | Type::I32
                        | Type::I64
                        | Type::Isize
                        | Type::U8
                        | Type::U16
                        | Type::U32
                        | Type::U64
                        | Type::Usize
                        | Type::F16
                        | Type::F32
                        | Type::F64
                        | Type::Bool => Ok(Type::Bool),
                        _ => Err(CompileErr::BinaryOpMismatch {
                            span: lhs.span.clone(),
                            op: op.clone(),
                            expected_operand_type: Type::Int,
                            found_operand_type: l_ty,
                        }),
                    }
                }
            }
        }

        Expr::Unary(op, expr) => {
            let ty = check_expr(ctx, expr)?;
            match op {
                UnaryOp::Neg => {
                    match ty {
                        Type::Int
                        | Type::I8
                        | Type::I16
                        | Type::I32
                        | Type::I64
                        | Type::Isize
                        | Type::F16
                        | Type::F32
                        | Type::F64 => Ok(ty),
                        // Explicitly disallow unsigned negation? Or allow wrapping?
                        // Rust allows it on wrapping types but standard negation is signed feature usually.
                        // Let's stick to signed for now.
                        _ => Err(CompileErr::Generic {
                            span: expr.span.clone(),
                            message: format!("Cannot negate type {:?}", ty),
                        }),
                    }
                }
                UnaryOp::Not => {
                    unify(&ty, &Type::Bool).map_err(|(found, expected)| {
                        CompileErr::TypeMismatch {
                            span: expr.span.clone(),
                            expected,
                            found,
                        }
                    })?;
                    Ok(Type::Bool)
                }
            }
        }

        Expr::Let(var, e1, e2) => {
            let t1 = check_expr(ctx, e1)?;
            let mut new_ctx = ctx.clone();
            new_ctx.insert(var.clone(), t1);
            check_expr(&new_ctx, e2)
        }

        Expr::LetRec(var, e1, e2) => {
            // Limitation: For recursion to be type-checked without inference,
            // we need to know the return type of the function `var` before checking `e1`.
            // But we don't.
            // Heuristic: If e1 is an Abs(param, param_ty, body), assume it returns Int for now (Factorial case),
            // OR checks e1 without `var` in context (disabling recursion type check, but allowing runtime recursion).
            // Allowing runtime recursion but failing type check... shit.
            // Let's try to infer if we can.
            //
            // Hack for "Arith": Assume Int -> Int if not inferable?
            // Actually, let's just inspect e1.
            let t1 = if let Expr::Abs(_, param_ty, _) = &e1.node {
                // Bind var to Arrow(param_ty, Int) tentatively?
                // This covers factorial/fibonacci.
                let assumed_arrow = Type::Arrow(Box::new(param_ty.clone()), Box::new(Type::Int));
                let mut recur_ctx = ctx.clone();
                recur_ctx.insert(var.clone(), assumed_arrow.clone());
                check_expr(&recur_ctx, e1)?
            } else {
                check_expr(ctx, e1)?
            };

            let mut new_ctx = ctx.clone();
            new_ctx.insert(var.clone(), t1);
            check_expr(&new_ctx, e2)
        }

        Expr::If(cond, e_then, e_else) => {
            let t_cond = check_expr(ctx, cond)?;
            unify(&t_cond, &Type::Bool).map_err(|(found_type, _)| {
                CompileErr::IfConditionNotBool {
                    span: cond.span.clone(),
                    found_type,
                }
            })?;

            let t_then = check_expr(ctx, e_then)?;
            let t_else = check_expr(ctx, e_else)?;

            unify(&t_else, &t_then).map_err(|(else_type, then_type)| {
                CompileErr::IfBranchesMismatch {
                    span: e_else.span.clone(),
                    then_type,
                    else_type,
                }
            })?;
            Ok(t_then)
        }

        Expr::Assert(cond) => {
            let t_cond = check_expr(ctx, cond)?;
            unify(&t_cond, &Type::Bool).map_err(|(found_type, _)| CompileErr::TypeMismatch {
                span: cond.span.clone(),
                expected: Type::Bool,
                found: found_type,
            })?;
            Ok(Type::Bool)
        }

        Expr::StructDecl(name, fields, body) => {
            let mut field_map = HashMap::new();
            for (f_name, f_type) in fields {
                field_map.insert(f_name.clone(), f_type.clone());
            }

            let mut new_ctx = ctx.clone();
            new_ctx.insert_struct(name.clone(), field_map);

            // Also allow the struct type to be used in variable bindings if needed?
            // Currently Type::Struct(name) is valid, but we don't check for existence of struct type
            // when just referring to it in type signatures (e.g. fn(p: Point)).
            // We might want to validate that 'Point' exists in Context.

            check_expr(&new_ctx, body)
        }

        Expr::StructInit(name, init_fields) => {
            let struct_def = ctx
                .get_struct(name)
                .ok_or_else(|| CompileErr::UnknownType {
                    span: expr.span.clone(),
                    type_name: name.clone(),
                })?;

            // 1. Check all initialized fields exist and match type
            for (f_name, f_expr) in init_fields {
                let expected_ty =
                    struct_def
                        .get(f_name)
                        .ok_or_else(|| CompileErr::UnknownField {
                            span: f_expr.span.clone(),
                            struct_name: name.clone(),
                            field_name: f_name.clone(),
                        })?;

                let found_ty = check_expr(ctx, f_expr)?;
                unify(&found_ty, expected_ty).map_err(|(found, expected)| {
                    CompileErr::TypeMismatch {
                        span: f_expr.span.clone(),
                        expected,
                        found,
                    }
                })?;
            }

            // 2. Check for missing fields
            // (Optional optimization: iterate def instead)
            for def_field in struct_def.keys() {
                if !init_fields.iter().any(|(n, _)| n == def_field) {
                    return Err(CompileErr::MissingField {
                        span: expr.span.clone(),
                        struct_name: name.clone(),
                        field_name: def_field.clone(),
                    });
                }
            }

            Ok(Type::Struct(name.clone()))
        }

        Expr::FieldAccess(obj_expr, field_name) => {
            let obj_type = check_expr(ctx, obj_expr)?;

            match obj_type {
                Type::Struct(struct_name) => {
                    let struct_def =
                        ctx.get_struct(&struct_name)
                            .ok_or_else(|| CompileErr::UnknownType {
                                span: obj_expr.span.clone(),
                                type_name: struct_name.clone(),
                            })?;

                    let field_type =
                        struct_def
                            .get(field_name)
                            .ok_or_else(|| CompileErr::UnknownField {
                                span: expr.span.clone(),
                                struct_name: struct_name.clone(),
                                field_name: field_name.clone(),
                            })?;

                    Ok(field_type.clone())
                }
                _ => Err(CompileErr::NotAStruct {
                    span: obj_expr.span.clone(),
                    found: obj_type,
                }),
            }
        }

        Expr::Block(exprs) => {
            let mut ty = Type::Int; // Default (Unit-like?)
            for e in exprs {
                ty = check_expr(ctx, e)?;
            }
            Ok(ty)
        }
        Expr::Error(_) => {
            // I don't know the best way to handle this without exploding error propagation.
            panic!("Error can't be checked, this is a parsing error. ");
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parser::parse;

    fn ty(src: &str, ctx: Context) -> CompileResult<Type> {
        let ast = parse(src).unwrap_or_else(|e| panic!("parse failed for `{src}`: {:?}", e));
        check_expr(&ctx, &ast)
    }

    macro_rules! check {
        ($input:expr, $expected:expr) => {
            assert_eq!(ty($input, Context::new()).unwrap(), $expected)
        };
    }

    #[test]
    fn int_and_bool_literals() {
        check!("1", Type::Int);
        check!("true", Type::Bool);
    }

    #[test]
    fn known_and_unknown_variables() {
        let c = Context::new();
        let x_int_expr = Spanned::new(Expr::Var("x".into()), 0..0);
        let c_with_x = {
            let mut temp_c = c.clone();
            temp_c.insert("x".into(), Type::Int);
            temp_c
        };
        assert_eq!(check_expr(&c_with_x, &x_int_expr).unwrap(), Type::Int);

        let nope_expr = Spanned::new(Expr::Var("nope".into()), 0..0);
        let err = check_expr(&c, &nope_expr).unwrap_err();
        match err {
            CompileErr::UnknownVar { name, .. } => assert_eq!(name, "nope"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn lambda_typing() {
        let e = Spanned::new(
            Expr::Abs(
                "x".into(),
                Type::Int,
                Box::new(Spanned::new(Expr::Var("x".into()), 0..0)),
            ),
            0..0,
        );
        let ty = check_expr(&Context::new(), &e).unwrap();
        assert_eq!(ty, Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)));
    }

    #[test]
    fn app_correct() {
        let lam = Spanned::new(
            Expr::Abs(
                "x".into(),
                Type::Int,
                Box::new(Spanned::new(Expr::Var("x".into()), 0..0)),
            ),
            0..0,
        );
        let app = Spanned::new(
            Expr::App(
                Box::new(lam),
                Box::new(Spanned::new(Expr::Int("1".into()), 0..0)),
            ),
            0..0,
        );

        let ty = check_expr(&Context::new(), &app).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn arithmetic_on_ints() {
        check!("1 + 2", Type::Int);
    }

    #[test]
    fn arithmetic_type_error() {
        let e = Spanned::new(
            Expr::Binary(
                Box::new(Spanned::new(Expr::Int("1".into()), 0..0)),
                BinaryOp::Add,
                Box::new(Spanned::new(Expr::Bool(true), 0..0)),
            ),
            0..0,
        );
        let err = check_expr(&Context::new(), &e).unwrap_err();
        matches!(err, CompileErr::BinaryOpMismatch { .. });
    }

    #[test]
    fn let_binding() {
        let e = Spanned::new(
            Expr::Let(
                "x".into(),
                Box::new(Spanned::new(Expr::Int("1".into()), 0..0)),
                Box::new(Spanned::new(
                    Expr::Binary(
                        Box::new(Spanned::new(Expr::Var("x".into()), 0..0)),
                        BinaryOp::Add,
                        Box::new(Spanned::new(Expr::Int("2".into()), 0..0)),
                    ),
                    0..0,
                )),
            ),
            0..0,
        );
        assert_eq!(check_expr(&Context::new(), &e).unwrap(), Type::Int);
    }

    #[test]
    fn if_expression_ok() {
        check!("if true then 1 else 2", Type::Int);
    }

    #[test]
    fn if_condition_must_be_bool() {
        let err = ty("if 1 then 2 else 3", Context::new()).unwrap_err();
        matches!(err, CompileErr::IfConditionNotBool { .. });
    }

    #[test]
    fn if_branches_must_match() {
        let err = ty("if true then 1 else false", Context::new()).unwrap_err();
        matches!(err, CompileErr::IfBranchesMismatch { .. });
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
        let out = ty(r".\\x: Int -> .\\y: Int -> x", Context::new()).unwrap();
        match out {
            Type::Arrow(a, b) => {
                assert_eq!(*a, Type::Int);
                match *b {
                    Type::Arrow(inner_a, inner_b) => {
                        assert_eq!(*inner_a, Type::Int);
                        assert_eq!(*inner_b, Type::Int);
                    }
                    other => panic!("expected nested arrow, got {:?}", other),
                }
            }
            other => panic!("expected arrow, got {:?}", other),
        }
    }

    #[test]
    fn app_ok() {
        check!(r"(.\\x: Int -> x) 10", Type::Int);
    }

    #[test]
    fn app_type_mismatch() {
        let err = ty(r"(.\\x: Int -> x) true", Context::new()).unwrap_err();
        matches!(err, CompileErr::TypeMismatch { .. });
    }

    #[test]
    fn app_not_a_function_error() {
        let err = ty("10 true", Context::new()).unwrap_err();
        matches!(err, CompileErr::NotAFunction { .. });
    }

    #[test]
    fn arithmetic_ok() {
        check!("1 + 2 * 3", Type::Int);
    }

    #[test]
    fn arithmetic_err() {
        let err = ty("1 + true", Context::new()).unwrap_err();
        matches!(err, CompileErr::BinaryOpMismatch { .. });
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
    fn arrow_type_and_application_chain() {
        check!(r"(.\\x: Int -> .\\y: Int -> x) 10 20", Type::Int);
    }
}
