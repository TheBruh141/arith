use crate::syntax::ast::{BinaryOp, Expr, UnaryOp};
use crate::vm::core::OpCode;

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
            Expr::Int(s) => {
                let n = s
                    .parse::<num_bigint::BigInt>()
                    .expect("Invalid BigInt literal");
                self.emit(OpCode::PushInt(n));
            }
            Expr::Bool(b) => self.emit(OpCode::PushBool(*b)),

            Expr::LitI8(n) => self.emit(OpCode::PushI8(*n)),
            Expr::LitI16(n) => self.emit(OpCode::PushI16(*n)),
            Expr::LitI32(n) => self.emit(OpCode::PushI32(*n)),
            Expr::LitI64(n) => self.emit(OpCode::PushI64(*n)),
            Expr::LitIsize(n) => self.emit(OpCode::PushIsize(*n)),

            Expr::LitU8(n) => self.emit(OpCode::PushU8(*n)),
            Expr::LitU16(n) => self.emit(OpCode::PushU16(*n)),
            Expr::LitU32(n) => self.emit(OpCode::PushU32(*n)),
            Expr::LitU64(n) => self.emit(OpCode::PushU64(*n)),
            Expr::LitUsize(n) => self.emit(OpCode::PushUsize(*n)),

            Expr::LitF16(n) => self.emit(OpCode::PushF16(*n)),
            Expr::LitF32(n) => self.emit(OpCode::PushF32(*n)),
            Expr::LitF64(n) => self.emit(OpCode::PushF64(*n)),

            Expr::Unary(op, expr) => {
                self.compile(&expr.node);
                match op {
                    UnaryOp::Neg => self.emit(OpCode::Neg),
                    UnaryOp::Not => self.emit(OpCode::Not),
                }
            }

            Expr::Binary(lhs, op, rhs) => {
                self.compile(&lhs.node);
                self.compile(&rhs.node);
                match op {
                    BinaryOp::Add => self.emit(OpCode::Add),
                    BinaryOp::Sub => self.emit(OpCode::Sub),
                    BinaryOp::Mul => self.emit(OpCode::Mul),
                    BinaryOp::Div => self.emit(OpCode::Div),
                    BinaryOp::Equals => self.emit(OpCode::Eq),
                    BinaryOp::NotEquals => {
                        self.emit(OpCode::Eq);
                        self.emit(OpCode::Not);
                    }
                    BinaryOp::LessThan => self.emit(OpCode::Lt),
                    BinaryOp::GreaterThan => self.emit(OpCode::Gt),
                    BinaryOp::LessThanEquals => self.emit(OpCode::Leq),
                    BinaryOp::GreaterThanEquals => self.emit(OpCode::Geq),
                }
            }

            Expr::Var(name) => self.emit(OpCode::Load(name.clone())),

            Expr::Let(name, val_expr, body_expr) => {
                // lhs push
                self.compile(&val_expr.node);
                self.emit(OpCode::Store(name.clone()));

                // rhs push
                self.compile(&body_expr.node);
                self.emit(OpCode::PopBinding(name.clone()));
            }

            Expr::LetRec(name, val_expr, body_expr) => {
                if let Expr::Abs(param, _, fn_body) = &val_expr.node {
                    // Special handling to emit MakeRecursiveClosure
                    let jump_over_idx = self.code.len();
                    self.emit(OpCode::Jump(0)); // Placeholder

                    let fn_start = self.code.len();
                    self.compile(&fn_body.node);
                    self.emit(OpCode::Return);

                    let after_fn = self.code.len();
                    self.code[jump_over_idx] = OpCode::Jump(after_fn);

                    // Analyze free variables for the recursive closure
                    let mut bound = vec![param.clone(), name.clone()];
                    let mut captures = Self::analyze_free_vars(&fn_body.node, &mut bound);
                    captures.sort();
                    captures.dedup();

                    self.emit(OpCode::MakeRecursiveClosure {
                        addr: fn_start,
                        param: param.clone(),
                        rec_name: name.clone(),
                        captures,
                    });

                    // Store the closure
                    self.emit(OpCode::Store(name.clone()));

                    // Compile body
                    self.compile(&body_expr.node);

                    // Clean up
                    self.emit(OpCode::PopBinding(name.clone()));
                } else {
                    // Fallback to normal let if not a function (e.g. let rec x = 1 is just let x = 1)
                    self.compile(&val_expr.node);
                    self.emit(OpCode::Store(name.clone()));
                    self.compile(&body_expr.node);
                    self.emit(OpCode::PopBinding(name.clone()));
                }
            }

            Expr::If(cond, then_expr, else_expr) => {
                self.compile(&cond.node);

                let else_jump_idx = self.code.len();
                self.emit(OpCode::JumpIfFalse(0)); // Placeholder

                self.compile(&then_expr.node);

                let end_jump_idx = self.code.len();
                self.emit(OpCode::Jump(0)); // Placeholder

                // Patch JumpIfFalse to point here (start of Else)
                let else_start = self.code.len();
                self.code[else_jump_idx] = OpCode::JumpIfFalse(else_start);

                self.compile(&else_expr.node);

                // Patch Jump to point here (End)
                let end_pos = self.code.len();
                self.code[end_jump_idx] = OpCode::Jump(end_pos);
            }

            Expr::Abs(param, _, body) => {
                let jump_over_idx = self.code.len();
                self.emit(OpCode::Jump(0)); // Placeholder

                let fn_start = self.code.len();
                // Compile body
                self.compile(&body.node);
                // When a function returns, the result is on the stack.
                // We do NOT need to emit PopBinding here for the parameter,
                // because the VM tears down the entire CallFrame (including all locals)
                // automatically on OpCode::Return.
                self.emit(OpCode::Return);

                let after_fn = self.code.len();
                self.code[jump_over_idx] = OpCode::Jump(after_fn);

                // Analyze free variables to capture
                let mut bound = vec![param.clone()];
                let mut captures = Self::analyze_free_vars(&body.node, &mut bound);
                captures.sort();
                captures.dedup();

                self.emit(OpCode::MakeClosure {
                    addr: fn_start,
                    param: param.clone(),
                    captures,
                });
            }

            // ... (App, Assert, Block, etc.)
            _ => self.compile_rest(expr),
        }
    }

    fn compile_rest(&mut self, expr: &Expr) {
        match expr {
            Expr::App(func, arg) => {
                self.compile(&func.node); // Pushes Closure
                self.compile(&arg.node); // Pushes Argument
                self.emit(OpCode::Call);
            }

            Expr::Assert(e) => {
                self.compile(&e.node);
                self.emit(OpCode::Assert);
            }

            Expr::Block(exprs) => {
                let len = exprs.len();
                for (i, e) in exprs.iter().enumerate() {
                    self.compile(&e.node);
                    // Pop result of expression unless it's the last one
                    if i < len - 1 {
                        self.emit(OpCode::Pop);
                    }
                }
                if len == 0 {
                    // Empty block -> Void/Unit? Push 0
                    self.emit(OpCode::PushInt(num_bigint::BigInt::from(0)));
                }
            }
            Expr::StructDef {
                name: _,
                fields: _,
                body,
            } => {
                // Struct declaration is purely checking/metadata.
                // At runtime, we just execute the body.
                self.compile(&body.node);
            }

            Expr::StructInit(name, fields) => {
                let mut field_names = Vec::new();
                for (f_name, f_expr) in fields {
                    self.compile(&f_expr.node);
                    field_names.push(f_name.clone());
                }
                self.emit(OpCode::MakeStruct {
                    name: name.clone(),
                    fields: field_names,
                });
            }

            Expr::FieldAccess(obj, field) => {
                self.compile(&obj.node);
                self.emit(OpCode::GetField(field.clone()));
            }

            _ => {} // Should be covered
        }
    }

    fn emit(&mut self, op: OpCode) {
        self.code.push(op);
    }

    // --- Static Analysis Helpers ---

    /// Analyzes an expression to find "free variables" (variables used but not defined in the expression).
    /// `bound` is the set of variables defined mainly by function parameters or let-bindings wrapping the expression.
    fn analyze_free_vars(expr: &Expr, bound: &mut Vec<String>) -> Vec<String> {
        match expr {
            Expr::Var(name) => {
                if !bound.contains(name) {
                    vec![name.clone()]
                } else {
                    vec![]
                }
            }
            Expr::Abs(param, _, body) => {
                bound.push(param.clone());
                let free = Self::analyze_free_vars(&body.node, bound);
                bound.pop();
                free
            }
            Expr::App(func, arg) => {
                let mut free = Self::analyze_free_vars(&func.node, bound);
                free.extend(Self::analyze_free_vars(&arg.node, bound));
                free
            }
            Expr::Binary(lhs, _, rhs) => {
                let mut free = Self::analyze_free_vars(&lhs.node, bound);
                free.extend(Self::analyze_free_vars(&rhs.node, bound));
                free
            }
            Expr::Let(name, val, body) => {
                // val is evaluated in current scope
                let mut free = Self::analyze_free_vars(&val.node, bound);

                // body has `name` bound
                bound.push(name.clone());
                free.extend(Self::analyze_free_vars(&body.node, bound));
                bound.pop();
                free
            }
            Expr::LetRec(name, val, body) => {
                // LetRec binds `name` in BOTH val (for recursion) and body
                bound.push(name.clone());
                let mut free = Self::analyze_free_vars(&val.node, bound);
                free.extend(Self::analyze_free_vars(&body.node, bound));
                bound.pop();
                free
            }
            Expr::If(c, t, e) => {
                let mut free = Self::analyze_free_vars(&c.node, bound);
                free.extend(Self::analyze_free_vars(&t.node, bound));
                free.extend(Self::analyze_free_vars(&e.node, bound));
                free
            }
            Expr::Assert(e) => Self::analyze_free_vars(&e.node, bound),
            Expr::Block(exprs) => {
                let mut free = Vec::new();
                for e in exprs {
                    free.extend(Self::analyze_free_vars(&e.node, bound));
                }
                free
            }
            // Primitives have no variables
            _ => vec![],
        }
    }
}
