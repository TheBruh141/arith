use crate::checker::unify::unify;
use crate::compiler_errors::{CompileErr, CompileResult};
use crate::syntax::ast::Pattern;
use crate::syntax::ast::{BinaryOp, Expr, Spanned, Type, UnaryOp};

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TypeContext {
    structs: HashMap<String, HashMap<String, Type>>,
    enums: HashMap<String, HashMap<String, Vec<Type>>>,
}

impl Default for TypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn insert_struct(&mut self, name: String, fields: HashMap<String, Type>) {
        self.structs.insert(name, fields);
    }

    pub fn insert_enum(&mut self, name: String, variants: HashMap<String, Vec<Type>>) {
        self.enums.insert(name, variants);
    }

    pub fn get_struct(&self, name: &str) -> Option<&HashMap<String, Type>> {
        self.structs.get(name)
    }

    pub fn get_enum(&self, name: &str) -> Option<&HashMap<String, Vec<Type>>> {
        self.enums.get(name)
    }
}

#[derive(Clone, Debug)]
pub struct ValueContext {
    vars: HashMap<String, Type>,
}

impl Default for ValueContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueContext {
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

pub fn check_expr(
    types: &TypeContext,
    vals: &ValueContext,
    expr: &Spanned<Expr>,
) -> CompileResult<Type> {
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

        Expr::Var(name) => vals
            .get(name)
            .cloned()
            .ok_or_else(|| CompileErr::UnknownVar {
                span: expr.span.clone(),
                name: name.clone(),
            }),

        Expr::Abs(var, ty, body) => {
            let mut new_vals = vals.clone();
            new_vals.insert(var.clone(), ty.clone());

            let body_ty = check_expr(types, &mut new_vals, body)?;

            Ok(Type::Arrow(Box::new(ty.clone()), Box::new(body_ty)))
        }

        Expr::App(func, arg) => {
            let func_ty = check_expr(types, vals, func)?;
            let arg_ty = check_expr(types, vals, arg)?;

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
            let l_ty = check_expr(types, vals, lhs)?;
            let r_ty = check_expr(types, vals, rhs)?;

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
            let ty = check_expr(types, vals, expr)?;
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
            let t1 = check_expr(types, vals, e1)?;
            let mut new_vals = vals.clone();
            new_vals.insert(var.clone(), t1);
            check_expr(types, &mut new_vals, e2)
        }

        Expr::LetRec(var, e1, e2) => {
            // Same heuristic as you had, but we only mutate the value environment.
            let t1 = if let Expr::Abs(_, param_ty, _) = &e1.node {
                // Tentative arrow type for recursion
                let assumed_arrow = Type::Arrow(Box::new(param_ty.clone()), Box::new(Type::Int));
                let mut recur_vals = vals.clone();
                recur_vals.insert(var.clone(), assumed_arrow.clone());
                check_expr(types, &mut recur_vals, e1)?
            } else {
                check_expr(types, vals, e1)?
            };

            let mut new_vals = vals.clone();
            new_vals.insert(var.clone(), t1);
            check_expr(types, &mut new_vals, e2)
        }

        // --- If (unchanged except for ctx -> types/vals) ---
        Expr::If(cond, e_then, e_else) => {
            let t_cond = check_expr(types, vals, cond)?;
            unify(&t_cond, &Type::Bool).map_err(|(found_type, _)| {
                CompileErr::IfConditionNotBool {
                    span: cond.span.clone(),
                    found_type,
                }
            })?;

            let t_then = check_expr(types, vals, e_then)?;
            let t_else = check_expr(types, vals, e_else)?;

            unify(&t_else, &t_then).map_err(|(else_type, then_type)| {
                CompileErr::IfBranchesMismatch {
                    span: e_else.span.clone(),
                    then_type,
                    else_type,
                }
            })?;
            Ok(t_then)
        }

        // --- Assert ---
        Expr::Assert(cond) => {
            let t_cond = check_expr(types, vals, cond)?;
            unify(&t_cond, &Type::Bool).map_err(|(found_type, _)| CompileErr::TypeMismatch {
                span: cond.span.clone(),
                expected: Type::Bool,
                found: found_type,
            })?;
            Ok(Type::Bool)
        }

        // --- StructDef (adds a type; clone types, optionally clone vals for body) ---
        Expr::StructDef { name, fields, body } => {
            let mut field_map = HashMap::new();
            for (f_name, f_type) in fields {
                field_map.insert(f_name.clone(), f_type.clone());
            }

            let mut new_types = types.clone();
            new_types.insert_struct(name.clone(), field_map);

            // Body sees the same value environment, but if you want the struct def
            // not to leak into surrounding scopes, cloning types is necessary.
            let mut new_vals = vals.clone();
            check_expr(&new_types, &mut new_vals, body)
        }

        // --- StructInit (read-only types, value checking uses vals) ---
        Expr::StructInit(name, init_fields) => {
            let struct_def = types
                .get_struct(name)
                .ok_or_else(|| CompileErr::UnknownType {
                    span: expr.span.clone(),
                    type_name: name.clone(),
                })?;

            // 1. Check initialized fields exist and match types
            for (f_name, f_expr) in init_fields {
                let expected_ty =
                    struct_def
                        .get(f_name)
                        .ok_or_else(|| CompileErr::UnknownField {
                            span: f_expr.span.clone(),
                            struct_name: name.clone(),
                            field_name: f_name.clone(),
                        })?;

                let found_ty = check_expr(types, vals, f_expr)?;
                unify(&found_ty, expected_ty).map_err(|(found, expected)| {
                    CompileErr::TypeMismatch {
                        span: f_expr.span.clone(),
                        expected,
                        found,
                    }
                })?;
            }

            // 2. Check for missing fields
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

        // --- FieldAccess ---
        Expr::FieldAccess(obj_expr, field_name) => {
            let obj_type = check_expr(types, vals, obj_expr)?;

            match obj_type {
                Type::Struct(struct_name) => {
                    let struct_def =
                        types
                            .get_struct(&struct_name)
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

        // --- Block (keeps same vals; if you want block scoping, clone vals here) ---
        Expr::Block(exprs) => {
            let mut ty = Type::Int; // Default (Unit-like?)
            for e in exprs {
                ty = check_expr(types, vals, e)?;
            }
            Ok(ty)
        }

        // --- Error ---
        Expr::Error(_) => {
            panic!("Error can't be checked, this is a parsing error.");
        }

        // --- EnumDef (adds type) ---
        Expr::EnumDef {
            name,
            variants,
            body,
        } => {
            let mut variant_map = HashMap::new();
            for (v_name, v_types) in variants {
                variant_map.insert(v_name.clone(), v_types.clone());
            }

            let mut new_types = types.clone();
            new_types.insert_enum(name.clone(), variant_map);

            let mut new_vals = vals.clone();
            check_expr(&new_types, &mut new_vals, body)
        }

        // --- EnumInit ---
        Expr::EnumInit {
            enum_name,
            variant_name,
            values,
        } => {
            // 1. Check existence
            let enum_def = types.get_enum(enum_name).ok_or(CompileErr::UnknownType {
                span: expr.span.clone(),
                type_name: enum_name.clone(),
            })?;

            // 2. Variant exists?
            let expected_types =
                enum_def
                    .get(variant_name)
                    .ok_or_else(|| CompileErr::UnknownVariant {
                        span: expr.span.clone(),
                        enum_name: enum_name.clone(),
                        variant_name: variant_name.clone(),
                    })?;

            // 3. Arity
            if values.len() != expected_types.len() {
                return Err(CompileErr::ArityMismatch {
                    span: expr.span.clone(),
                    expected: expected_types.len(),
                    found: values.len(),
                });
            }

            // 4. Check types of arguments
            for (val, expected) in values.iter().zip(expected_types) {
                let found = check_expr(types, vals, val)?;
                unify(&found, expected).map_err(|(found_ty, expected_ty)| {
                    CompileErr::TypeMismatch {
                        span: val.span.clone(),
                        expected: expected_ty,
                        found: found_ty,
                    }
                })?;
            }

            Ok(Type::Enum(enum_name.clone()))
        }

        // --- Match (value-pattern binding uses cloned value context per arm) ---
        Expr::Match { value, arms } => {
            // 1. Check scrutinee
            let val_type = check_expr(types, vals, value)?;

            // 2. Must be enum currently
            let enum_name = match &val_type {
                Type::Enum(n) => n,
                _ => {
                    return Err(CompileErr::NotAnEnum {
                        span: value.span.clone(),
                        found: val_type,
                    });
                }
            };

            // 3. Arm return type handling
            let mut return_type: Option<Type> = None;

            for (pat, arm_expr) in arms {
                // clone only the value context for arm-local bindings
                let mut arm_vals = vals.clone();

                // Now check pattern — pattern checking may read types and mutate arm_vals
                check_pattern(types, &mut arm_vals, pat, &val_type)?;

                // Check body under types and arm-specific value env
                let arm_ty = check_expr(types, &mut arm_vals, arm_expr)?;

                if let Some(ref ret) = return_type {
                    unify(&arm_ty, ret).map_err(|(found, expected)| CompileErr::TypeMismatch {
                        span: arm_expr.span.clone(),
                        expected,
                        found,
                    })?;
                } else {
                    return_type = Some(arm_ty);
                }
            }

            Ok(return_type.unwrap_or(Type::Int))
        }
    }
}

/// Recursive function to check if a pattern matches the expected type,
/// and bind any variables found in the pattern to the context.
fn check_pattern(
    types: &TypeContext,
    vals: &mut ValueContext,
    pat: &Pattern,
    expected_type: &Type,
) -> CompileResult<()> {
    match pat {
        // 1. Variable pattern: binds a new value
        Pattern::Var(name) => {
            vals.insert(name.clone(), expected_type.clone());
            Ok(())
        }

        // 2. Wildcard: matches anything, binds nothing
        Pattern::Wildcard => Ok(()),

        // 3. Integer literal
        Pattern::LitInt(_) => {
            if *expected_type != Type::Int {
                return Err(CompileErr::PatternMismatch {
                    span: 0..0,
                    expected: expected_type.clone(),
                    found: "Integer Literal".to_string(),
                });
            }
            Ok(())
        }

        // 4. Enum pattern
        Pattern::EnumPat(enum_name, variant_name, sub_pats) => {
            // A. Expected type must be the same enum
            match expected_type {
                Type::Enum(name) if name == enum_name => {}
                _ => {
                    return Err(CompileErr::PatternMismatch {
                        span: 0..0,
                        expected: expected_type.clone(),
                        found: format!("Enum {}::{}", enum_name, variant_name),
                    });
                }
            }

            // B. Look up enum definition (TYPE info → TypeContext)
            let enum_def = types.get_enum(enum_name).ok_or(CompileErr::UnknownType {
                span: 0..0,
                type_name: enum_name.clone(),
            })?;

            let field_types = enum_def
                .get(variant_name)
                .ok_or(CompileErr::UnknownVariant {
                    span: 0..0,
                    enum_name: enum_name.clone(),
                    variant_name: variant_name.clone(),
                })?;

            // C. Arity check
            if sub_pats.len() != field_types.len() {
                return Err(CompileErr::ArityMismatch {
                    span: 0..0,
                    expected: field_types.len(),
                    found: sub_pats.len(),
                });
            }

            // D. Recurse
            for (sub_pat, sub_type) in sub_pats.iter().zip(field_types.iter()) {
                check_pattern(types, vals, sub_pat, sub_type)?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parser::parse;

    fn ty(src: &str, types: &TypeContext, vals: &ValueContext) -> CompileResult<Type> {
        let ast = parse(src).unwrap_or_else(|e| panic!("parse failed for `{src}`: {:?}", e));
        check_expr(types, vals, &ast)
    }

    macro_rules! check {
        ($input:expr, $expected:expr) => {{
            let types = TypeContext::new();
            let vals = ValueContext::new();
            assert_eq!(ty($input, &types, &vals).unwrap(), $expected);
        }};
    }

    #[test]
    fn int_and_bool_literals() {
        check!("1", Type::Int);
        check!("true", Type::Bool);
    }

    #[test]
    fn known_and_unknown_variables() {
        let types = TypeContext::new();

        let mut vals = ValueContext::new();
        vals.insert("x".into(), Type::Int);

        let x_expr = Spanned::new(Expr::Var("x".into()), 0..0);
        assert_eq!(check_expr(&types, &vals, &x_expr).unwrap(), Type::Int);

        let nope_expr = Spanned::new(Expr::Var("nope".into()), 0..0);
        let err = check_expr(&types, &ValueContext::new(), &nope_expr).unwrap_err();

        match err {
            CompileErr::UnknownVar { name, .. } => assert_eq!(name, "nope"),
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn lambda_typing() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let e = Spanned::new(
            Expr::Abs(
                "x".into(),
                Type::Int,
                Box::new(Spanned::new(Expr::Var("x".into()), 0..0)),
            ),
            0..0,
        );

        let ty = check_expr(&types, &vals, &e).unwrap();
        assert_eq!(ty, Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)));
    }

    #[test]
    fn app_correct() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

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

        let ty = check_expr(&types, &vals, &app).unwrap();
        assert_eq!(ty, Type::Int);
    }

    #[test]
    fn arithmetic_on_ints() {
        check!("1 + 2", Type::Int);
    }

    #[test]
    fn arithmetic_type_error() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let e = Spanned::new(
            Expr::Binary(
                Box::new(Spanned::new(Expr::Int("1".into()), 0..0)),
                BinaryOp::Add,
                Box::new(Spanned::new(Expr::Bool(true), 0..0)),
            ),
            0..0,
        );

        let err = check_expr(&types, &vals, &e).unwrap_err();
        matches!(err, CompileErr::BinaryOpMismatch { .. });
    }

    #[test]
    fn let_binding() {
        check!("let x = 1 in x + 2", Type::Int);
    }

    #[test]
    fn if_expression_ok() {
        check!("if true then 1 else 2", Type::Int);
    }

    #[test]
    fn if_condition_must_be_bool() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let err = ty("if 1 then 2 else 3", &types, &vals).unwrap_err();
        matches!(err, CompileErr::IfConditionNotBool { .. });
    }

    #[test]
    fn if_branches_must_match() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let err = ty("if true then 1 else false", &types, &vals).unwrap_err();
        matches!(err, CompileErr::IfBranchesMismatch { .. });
    }

    #[test]
    fn lambda_nested() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let out = ty(r".\\x: Int -> .\\y: Int -> x", &types, &vals).unwrap();

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
    fn application_chain() {
        check!(r"(.\\x: Int -> .\\y: Int -> x) 10 20", Type::Int);
    }

    // =========================
    // EXTENSIONS (new tests)
    // =========================

    #[test]
    fn lambda_does_not_leak_binding() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let e = parse(".\\x: Int -> x").unwrap();
        let _ = check_expr(&types, &vals, &e).unwrap();

        let x = Spanned::new(Expr::Var("x".into()), 0..0);
        let err = check_expr(&types, &vals, &x).unwrap_err();

        matches!(err, CompileErr::UnknownVar { .. });
    }

    #[test]
    fn match_arm_bindings_are_isolated() {
        let types = TypeContext::new();
        let vals = ValueContext::new();

        let src = r"
            enum Option { Some(Int), None }
            match Some(1) {
                Some(x) => x,
                None => 0
            }
        ";

        let ast = parse(src).unwrap();
        let ty = check_expr(&types, &vals, &ast).unwrap();

        assert_eq!(ty, Type::Int);
    }
}
