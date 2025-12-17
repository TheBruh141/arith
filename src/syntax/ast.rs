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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Lit(i64),
    Var(String),
    Binary(Box<TypeExpr>, BinaryOp, Box<TypeExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),

    // Literals
    Int(String), // BigInt stored as string initially
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
    LitF16(f32), // Half is tricky, store as f32 for AST? Or use half crate wrapper later.
    // For AST, f32 is fine storage if we don't lose precision (we might).
    // Actually, let's store f32/f64.
    LitF32(f32),
    LitF64(f64),

    Abs(String, Type, Box<Spanned<Expr>>),
    App(Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Binary(Box<Spanned<Expr>>, BinaryOp, Box<Spanned<Expr>>),
    Let(String, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    LetRec(String, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    If(Box<Spanned<Expr>>, Box<Spanned<Expr>>, Box<Spanned<Expr>>),
    Assert(Box<Spanned<Expr>>),
    Block(Vec<Spanned<Expr>>),
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
        }
    }
}
impl Expr {
    /// Pretty-print the AST with colors and indentation
    pub fn debug_ast(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        match self {
            Expr::Var(name) => format!("{pad}{}Var{}({})", CYAN, RESET, name),
            Expr::Int(n) => format!("{pad}{}Int{}({})", GREEN, RESET, n),
            Expr::Bool(b) => format!("{pad}{}Bool{}({})", YELLOW, RESET, b),
            Expr::LitU8(n) => format!("{pad}{}U8{}({})", GREEN, RESET, n),
            Expr::LitU16(n) => format!("{pad}{}U16{}({})", GREEN, RESET, n),
            Expr::LitU32(n) => format!("{pad}{}U32{}({})", GREEN, RESET, n),
            Expr::LitU64(n) => format!("{pad}{}U64{}({})", GREEN, RESET, n),
            Expr::LitUsize(n) => format!("{pad}{}Usize{}({})", GREEN, RESET, n),
            Expr::LitI8(n) => format!("{pad}{}I8{}({})", GREEN, RESET, n),
            Expr::LitI16(n) => format!("{pad}{}I16{}({})", GREEN, RESET, n),
            Expr::LitI32(n) => format!("{pad}{}I32{}({})", GREEN, RESET, n),
            Expr::LitI64(n) => format!("{pad}{}I64{}({})", GREEN, RESET, n),
            Expr::LitIsize(n) => format!("{pad}{}Isize{}({})", GREEN, RESET, n),
            Expr::LitF16(n) => format!("{pad}{}F16{}({})", GREEN, RESET, n),
            Expr::LitF32(n) => format!("{pad}{}F32{}({})", GREEN, RESET, n),
            Expr::LitF64(n) => format!("{pad}{}F64{}({})", GREEN, RESET, n),
            Expr::Assert(e) => format!(
                "{pad}{}Assert{}(\n{})",
                RED,
                RESET,
                e.node.debug_ast(indent + 1)
            ),
            Expr::Block(exprs) => {
                let lines: Vec<String> =
                    exprs.iter().map(|e| e.node.debug_ast(indent + 1)).collect();
                format!("{pad}{}Block{}(\n{})", MAGENTA, RESET, lines.join("\n"))
            }

            Expr::Abs(param, ty, body) => {
                format!(
                    "{pad}{}Abs{}({param}: {}{})\n{}{}",
                    MAGENTA,
                    RESET,
                    BLUE,
                    ty.debug_type(),
                    RESET,
                    body.node.debug_ast(indent + 1)
                )
            }
            Expr::App(func, arg) => {
                format!(
                    "{pad}{}App{}\n{}{}\n{}{}",
                    RED,
                    RESET,
                    func.node.debug_ast(indent + 1),
                    "",
                    arg.node.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::Binary(lhs, op, rhs) => {
                format!(
                    "{pad}{}Binary{}({})\n{}{}\n{}{}",
                    CYAN,
                    RESET,
                    op,
                    lhs.node.debug_ast(indent + 1),
                    "",
                    rhs.node.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::Let(name, expr, body) => {
                format!(
                    "{pad}{}Let{}({})\n{}{}\n{}{}",
                    BLUE,
                    RESET,
                    name,
                    expr.node.debug_ast(indent + 1),
                    "",
                    body.node.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::LetRec(name, expr, body) => {
                format!(
                    "{pad}{}LetRec{}({})\n{}{}\n{}{}",
                    BLUE,
                    RESET,
                    name,
                    expr.node.debug_ast(indent + 1),
                    "",
                    body.node.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::If(cond, then_br, else_br) => {
                format!(
                    "{pad}{}If{}\n{}{}\n{}{}\n{}{}",
                    RED,
                    RESET,
                    cond.node.debug_ast(indent + 1),
                    "",
                    then_br.node.debug_ast(indent + 1),
                    "",
                    else_br.node.debug_ast(indent + 1),
                    ""
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
