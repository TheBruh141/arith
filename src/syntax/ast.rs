use crate::compiler_errors::CompileErr;
use std::fmt::{Display, Formatter};
use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Spanned { node, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,

    // conditionals
    Equals,            // ==
    LessThan,          // <
    GreaterThan,       // >
    LessThanEquals,    // <=
    GreaterThanEquals, // =>
    NotEquals,         // !=
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int, // BigInt by default
    Bool,
    // Primitives
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F16,
    F32,
    F64,

    Arrow(Box<Type>, Box<Type>),
    Vector(Box<Type>, TypeExpr),
    Struct(String),
    Enum(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Lit(i64),
    Var(String),
    Binary(Box<TypeExpr>, BinaryOp, Box<TypeExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // !
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Int(String), // BigInt
    Bool(bool),

    // Primitives
    LitU8(u8),
    LitU16(u16),
    LitU32(u32),
    LitU64(u64),
    LitUsize(usize),

    LitI8(i8),
    LitI16(i16),
    LitI32(i32),
    LitI64(i64),
    LitIsize(isize),

    LitF16(f32), // Rust doesn't have f16 separate lit logic usually, stored as f32 in AST for now? Or keep string?
    // Parser converts to float.
    LitF32(f32),
    LitF64(f64),

    // Variables
    Var(String),

    // Operators
    Unary(UnaryOp, Box<Spanned<Expr>>),
    Binary(Box<Spanned<Expr>>, BinaryOp, Box<Spanned<Expr>>),

    // Control Flow
    Abs(String, Type, Box<Spanned<Expr>>),
    App(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Let(String, Box<Spanned<Expr>>, Box<Spanned<Expr>>), // let x = e1 in e2
    LetRec(String, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    If(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Assert(Box<Spanned<Expr>>),
    Block(Vec<Spanned<Expr>>),

    // Structs
    // Definition: struct Point { x: Int } in ...
    StructDef {
        name: String,
        fields: Vec<(String, Type)>,
        body: Box<Spanned<Expr>>,
    },
    // Initialization: Point { x: 10 }
    StructInit(String, Vec<(String, Spanned<Expr>)>),
    // Access: p.x
    FieldAccess(Box<Spanned<Expr>>, String),

    // --- Enums ---
    EnumDef {
        name: String,
        variants: Vec<(String, Vec<Type>)>,
        body: Box<Spanned<Expr>>,
    },
    EnumInit {
        enum_name: String,
        variant_name: String,
        values: Vec<Spanned<Expr>>,
    },
    Match {
        value: Box<Spanned<Expr>>,
        arms: Vec<(Pattern, Spanned<Expr>)>,
    },

    // Not a good time to see this...
    Error(String /*Reason / Error message */),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Var(String),                           // x
    LitInt(String),                        // 10
    EnumPat(String, String, Vec<Pattern>), // Shape::Circle(r)
    Wildcard,                              // _
}

#[derive(Debug)]
pub struct PartialParse {
    pub ast: Option<Spanned<Expr>>,
    pub errors: Vec<CompileErr>,
}

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "BinaryOp(Add)"),
            BinaryOp::Sub => write!(f, "BinaryOp(Sub)"),
            BinaryOp::Mul => write!(f, "BinaryOp(Mul)"),
            BinaryOp::Div => write!(f, "BinaryOp(Div)"),

            BinaryOp::Equals => write!(f, "BinaryOp(Equals)"),
            BinaryOp::LessThan => write!(f, "BinaryOp(LessThan)"),
            BinaryOp::GreaterThan => write!(f, "BinaryOp(GreaterThan)"),
            BinaryOp::LessThanEquals => write!(f, "BinaryOp(LessThanEquals)"),
            BinaryOp::GreaterThanEquals => write!(f, "BinaryOp(GreaterThanEquals"),
            BinaryOp::NotEquals => write!(f, "BinaryOp(NotEquals)"),
        }
    }
}
impl Expr {
    pub fn debug_ast(&self) -> String {
        self.debug_ast_inner("", true)
    }

    fn debug_ast_inner(&self, prefix: &str, last: bool) -> String {
        let branch = if last { "└─" } else { "├─" };
        let next_prefix = if last {
            format!("{prefix}   ")
        } else {
            format!("{prefix}│  ")
        };

        let head = format!("{prefix}{branch}");

        match self {
            Expr::Var(name) => format!("{head}{}Var{}({name})", CYAN, RESET),
            Expr::Int(n) => format!("{head}{}Int{}({n})", GREEN, RESET),
            Expr::Bool(b) => format!("{head}{}Bool{}({b})", YELLOW, RESET),

            Expr::Unary(op, expr) => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                format!(
                    "{head}{}Unary{}({})\n{}",
                    RED,
                    RESET,
                    op_str,
                    expr.node.debug_ast_inner(&next_prefix, true)
                )
            }

            Expr::Binary(lhs, op, rhs) => {
                format!(
                    "{head}{}Binary{}({op})\n{}\n{}",
                    CYAN,
                    RESET,
                    lhs.node.debug_ast_inner(&next_prefix, false),
                    rhs.node.debug_ast_inner(&next_prefix, true),
                )
            }

            Expr::App(func, arg) => {
                format!(
                    "{head}{}App{}\n{}\n{}",
                    RED,
                    RESET,
                    func.node.debug_ast_inner(&next_prefix, false),
                    arg.node.debug_ast_inner(&next_prefix, true),
                )
            }

            Expr::Block(exprs) => {
                let mut out = format!("{head}{}Block{}", MAGENTA, RESET);
                for (i, e) in exprs.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&e.node.debug_ast_inner(&next_prefix, i == exprs.len() - 1));
                }
                out
            }

            Expr::Let(name, expr, body) => {
                format!(
                    "{head}{}Let{}({name})\n{}\n{}",
                    BLUE,
                    RESET,
                    expr.node.debug_ast_inner(&next_prefix, false),
                    body.node.debug_ast_inner(&next_prefix, true),
                )
            }

            Expr::LetRec(name, expr, body) => {
                format!(
                    "{head}{}LetRec{}({name})\n{}\n{}",
                    BLUE,
                    RESET,
                    expr.node.debug_ast_inner(&next_prefix, false),
                    body.node.debug_ast_inner(&next_prefix, true),
                )
            }

            Expr::If(cond, then_br, else_br) => {
                format!(
                    "{head}{}If{}\n{}\n{}\n{}",
                    RED,
                    RESET,
                    cond.node.debug_ast_inner(&next_prefix, false),
                    then_br.node.debug_ast_inner(&next_prefix, false),
                    else_br.node.debug_ast_inner(&next_prefix, true),
                )
            }

            Expr::Assert(e) => format!(
                "{head}{}Assert{}\n{}",
                RED,
                RESET,
                e.node.debug_ast_inner(&next_prefix, true)
            ),

            Expr::FieldAccess(base, field) => {
                format!(
                    "{head}{}FieldAccess{}({field})\n{}",
                    CYAN,
                    RESET,
                    base.node.debug_ast_inner(&next_prefix, true)
                )
            }

            Expr::StructInit(name, fields) => {
                let mut out = format!("{head}{}StructInit{}({name})", MAGENTA, RESET);
                for (i, (f, e)) in fields.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&format!(
                        "{next_prefix}{} {}{}
{} {}",
                        if i + 1 == fields.len() {
                            "└─"
                        } else {
                            "├─"
                        },
                        BLUE,
                        f,
                        RESET,
                        e.node.debug_ast_inner(&format!("{next_prefix}│  "), true)
                    ));
                }
                out
            }

            Expr::StructDef { name, fields, body } => {
                let mut out = format!("{head}{}StructDef{}({name})", MAGENTA, RESET);
                for (i, (f, ty)) in fields.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&format!(
                        "{next_prefix}{} {}{}: {} {}",
                        if i + 1 == fields.len() {
                            "└─"
                        } else {
                            "├─"
                        },
                        BLUE,
                        f,
                        RESET,
                        ty.debug_type()
                    ));
                }
                out.push('\n');
                out.push_str(&body.node.debug_ast_inner(&next_prefix, true));
                out
            }

            Expr::EnumDef {
                name,
                variants,
                body,
            } => {
                let mut out = format!("{head}{}EnumDef{}({name})", MAGENTA, RESET);
                for (i, (v, args)) in variants.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&format!(
                        "{next_prefix}{} {}{}({}) {}",
                        if i + 1 == variants.len() {
                            "└─"
                        } else {
                            "├─"
                        },
                        BLUE,
                        v,
                        RESET,
                        args.iter()
                            .map(|t| t.debug_type())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                out.push('\n');
                out.push_str(&body.node.debug_ast_inner(&next_prefix, true));
                out
            }

            Expr::EnumInit {
                enum_name,
                variant_name,
                values,
            } => {
                let mut out = format!(
                    "{head}{}EnumInit{}({enum_name}::{variant_name})",
                    MAGENTA, RESET
                );
                for (i, v) in values.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&v.node.debug_ast_inner(&next_prefix, i + 1 == values.len()));
                }
                out
            }

            Expr::Match { value, arms } => {
                let mut out = format!("{head}{}Match{}", RED, RESET);
                out.push('\n');
                out.push_str(&value.node.debug_ast_inner(&next_prefix, false));
                for (i, (pat, expr)) in arms.iter().enumerate() {
                    out.push('\n');
                    out.push_str(&format!(
                        "{next_prefix}{} {}Arm{}({pat:?})",
                        if i + 1 == arms.len() {
                            "└─"
                        } else {
                            "├─"
                        },
                        CYAN,
                        RESET
                    ));
                    out.push('\n');
                    out.push_str(
                        &expr
                            .node
                            .debug_ast_inner(&format!("{next_prefix}│  "), true),
                    );
                }
                out
            }

            Expr::Error(msg) => format!("{head}{}Error{}({msg})", RED, RESET),
            Expr::LitU8(n) => {
                format!("{head}{}LitU8{}({n})", GREEN, RESET)
            }
            Expr::LitU16(n) => {
                format!("{head}{}LitU16{}({n})", GREEN, RESET)
            }
            Expr::LitU32(n) => {
                format!("{head}{}LitU32{}({n})", GREEN, RESET)
            }
            Expr::LitU64(n) => {
                format!("{head}{}LitU64{}({n})", GREEN, RESET)
            }
            Expr::LitUsize(n) => {
                format!("{head}{}LitUsize{}({n})", GREEN, RESET)
            }
            Expr::LitI8(n) => {
                format!("{head}{}LitI8{}({n})", GREEN, RESET)
            }
            Expr::LitI16(n) => {
                format!("{head}{}LitI16{}({n})", GREEN, RESET)
            }
            Expr::LitI32(n) => {
                format!("{head}{}LitI32{}({n})", GREEN, RESET)
            }
            Expr::LitI64(n) => {
                format!("{head}{}LitI64{}({n})", GREEN, RESET)
            }
            Expr::LitIsize(n) => {
                format!("{head}{}LitIsize{}({n})", GREEN, RESET)
            }
            Expr::LitF16(n) => {
                format!("{head}{}LitF16{}({n})", GREEN, RESET)
            }
            Expr::LitF32(n) => {
                format!("{head}{}LitF32{}({n})", GREEN, RESET)
            }
            Expr::LitF64(n) => {
                format!("{head}{}LitF64{}({n})", GREEN, RESET)
            }
            Expr::Abs(param, ty, body) => {
                format!(
                    "{head}{}Abs{}({}{}:{}{})\n{}",
                    MAGENTA,
                    RESET,
                    BLUE,
                    param,
                    ty.debug_type(),
                    RESET,
                    body.node.debug_ast_inner(&next_prefix, true),
                )
            }
        }
    }
}

impl Type {
    pub fn debug_type(&self) -> String {
        match self {
            Type::Int => format!("{}Int{}", GREEN, RESET),
            Type::Bool => format!("{}Bool{}", YELLOW, RESET),
            Type::I8 => format!("{}i8{}", GREEN, RESET),
            Type::I16 => format!("{}i16{}", GREEN, RESET),
            Type::I32 => format!("{}i32{}", GREEN, RESET),
            Type::I64 => format!("{}i64{}", GREEN, RESET),
            Type::Isize => format!("{}isize{}", GREEN, RESET),
            Type::U8 => format!("{}u8{}", GREEN, RESET),
            Type::U16 => format!("{}u16{}", GREEN, RESET),
            Type::U32 => format!("{}u32{}", GREEN, RESET),
            Type::U64 => format!("{}u64{}", GREEN, RESET),
            Type::Usize => format!("{}usize{}", GREEN, RESET),
            Type::F16 => format!("{}f16{}", GREEN, RESET),
            Type::F32 => format!("{}f32{}", GREEN, RESET),
            Type::F64 => format!("{}f64{}", GREEN, RESET),

            Type::Arrow(from, to) => format!(
                "{}({} -> {}){}",
                MAGENTA,
                from.debug_type(),
                to.debug_type(),
                RESET
            ),
            Type::Vector(inner, size) => format!(
                "{}Vector<{}, {}>{}{}",
                CYAN,
                inner.debug_type(),
                size.debug_type(),
                RESET,
                ""
            ),
            Type::Struct(name) => format!("{}Struct \"{}\"{}", CYAN, name, RESET),
            Type::Enum(name) => format!("{}Enum \"{}\"{}", CYAN, name, RESET),
        }
    }
}

impl TypeExpr {
    pub fn debug_type(&self) -> String {
        match self {
            TypeExpr::Lit(n) => format!("{}{}{}", BLUE, n, RESET),
            TypeExpr::Var(name) => format!("{}{}{}", CYAN, name, RESET),
            TypeExpr::Binary(lhs, op, rhs) => format!(
                "{}({} {:?} {}){}",
                MAGENTA,
                lhs.debug_type(),
                op,
                rhs.debug_type(),
                RESET
            ),
        }
    }
}
