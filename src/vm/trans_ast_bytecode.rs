use crate::syntax::ast::{BinaryOp, Expr};
use crate::vm::vm::OpCode;

pub struct Compiler {
    pub code: Vec<OpCode>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self { code: Vec::new() }
    }

    pub fn compile(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(n) => self.emit(OpCode::PushInt(*n)),
            Expr::Bool(b) => self.emit(OpCode::PushBool(*b)),

            Expr::Binary(lhs, op, rhs) => {
                self.compile(lhs);
                self.compile(rhs);
                match op {
                    BinaryOp::Add => self.emit(OpCode::Add),
                    BinaryOp::Sub => self.emit(OpCode::Sub),
                    BinaryOp::Mul => self.emit(OpCode::Mul),
                    BinaryOp::Div => self.emit(OpCode::Div),
                    BinaryOp::Equals => self.emit(OpCode::Eq),
                    BinaryOp::LessThan => self.emit(OpCode::Lt),
                    BinaryOp::GreaterThan => self.emit(OpCode::Gt),
                    BinaryOp::LessThanEquals => self.emit(OpCode::Leq),
                    BinaryOp::GreaterThanEquals => self.emit(OpCode::Geq),
                }
            }

            Expr::Var(name) => self.emit(OpCode::Load(name.clone())),

            Expr::Let(name, val_expr, body_expr) => {
                // lhs push
                self.compile(val_expr);
                self.emit(OpCode::Store(name.clone()));

                // rhs push
                self.compile(body_expr);
                self.emit(OpCode::PopBinding(name.clone()));
            }

            Expr::If(cond, then_expr, else_expr) => {
                self.compile(cond);

                let else_jump_idx = self.code.len();
                self.emit(OpCode::JumpIfFalse(0)); // Placeholder

                self.compile(then_expr);

                let end_jump_idx = self.code.len();
                self.emit(OpCode::Jump(0)); // Placeholder

                // Patch JumpIfFalse to point here (start of Else)
                let else_start = self.code.len();
                self.code[else_jump_idx] = OpCode::JumpIfFalse(else_start);

                self.compile(else_expr);

                // Patch Jump to point here (End)
                let end_pos = self.code.len();
                self.code[end_jump_idx] = OpCode::Jump(end_pos);
            }

            Expr::Abs(param, _, body) => {
                let jump_over_idx = self.code.len();
                self.emit(OpCode::Jump(0)); // Placeholder

                let fn_start = self.code.len();
                self.compile(body);
                // When a function returns, the result is on the stack.
                // We do NOT need to emit PopBinding here for the parameter,
                // because the VM tears down the entire CallFrame (including all locals)
                // automatically on OpCode::Return.
                self.emit(OpCode::Return);

                let after_fn = self.code.len();
                self.code[jump_over_idx] = OpCode::Jump(after_fn);

                self.emit(OpCode::MakeClosure {
                    addr: fn_start,
                    param: param.clone(),
                });
            }

            Expr::App(func, arg) => {
                self.compile(func); // Pushes Closure
                self.compile(arg); // Pushes Argument
                self.emit(OpCode::Call);
            }
        }
    }

    fn emit(&mut self, op: OpCode) {
        self.code.push(op);
    }
}
