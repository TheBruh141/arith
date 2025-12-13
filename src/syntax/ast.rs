use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
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
    Int(i64),
    Bool(bool),
    Abs(String, Type, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
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
            Expr::Abs(param, ty, body) => {
                format!(
                    "{pad}{}Abs{}({param}: {}{})\n{}{}",
                    MAGENTA,
                    RESET,
                    BLUE,
                    ty.debug_type(),
                    RESET,
                    body.debug_ast(indent + 1)
                )
            }
            Expr::App(func, arg) => {
                format!(
                    "{pad}{}App{}\n{}{}\n{}{}",
                    RED,
                    RESET,
                    func.debug_ast(indent + 1),
                    "",
                    arg.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::Binary(lhs, op, rhs) => {
                format!(
                    "{pad}{}Binary{}({})\n{}{}\n{}{}",
                    CYAN,
                    RESET,
                    op,
                    lhs.debug_ast(indent + 1),
                    "",
                    rhs.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::Let(name, expr, body) => {
                format!(
                    "{pad}{}Let{}({})\n{}{}\n{}{}",
                    BLUE,
                    RESET,
                    name,
                    expr.debug_ast(indent + 1),
                    "",
                    body.debug_ast(indent + 1),
                    ""
                )
            }
            Expr::If(cond, then_br, else_br) => {
                format!(
                    "{pad}{}If{}\n{}{}\n{}{}\n{}{}",
                    RED,
                    RESET,
                    cond.debug_ast(indent + 1),
                    "",
                    then_br.debug_ast(indent + 1),
                    "",
                    else_br.debug_ast(indent + 1),
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
