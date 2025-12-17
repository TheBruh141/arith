use std::collections::HashMap;
use std::fmt;

use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(BigInt), // Default arbitrary precision
    Bool(bool),

    // Primitives
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Isize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Usize(usize),
    F16(f32), // Store as f32 for now
    F32(f32),
    F64(f64),

    Closure {
        addr: usize,
        param: String,
        env: HashMap<String, Value>,
        rec_name: Option<String>,
    },

    // Structs
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I8(n) => write!(f, "{}i8", n),
            Value::I16(n) => write!(f, "{}i16", n),
            Value::I32(n) => write!(f, "{}i32", n),
            Value::I64(n) => write!(f, "{}i64", n),
            Value::Isize(n) => write!(f, "{}isize", n),
            Value::U8(n) => write!(f, "{}u8", n),
            Value::U16(n) => write!(f, "{}u16", n),
            Value::U32(n) => write!(f, "{}u32", n),
            Value::U64(n) => write!(f, "{}u64", n),
            Value::Usize(n) => write!(f, "{}usize", n),
            Value::F16(n) => write!(f, "{}f16", n),
            Value::F32(n) => write!(f, "{}f32", n),
            Value::F64(n) => write!(f, "{}f64", n),
            Value::Closure { .. } => write!(f, "<function>"),
            // TODO: Pretty print struct fields
            Value::Struct { name, .. } => write!(f, "<struct {}>", name),
        }
    }
}

// --- Bytecode Instructions ---

#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack manipulation
    PushInt(BigInt),
    PushBool(bool),

    PushI8(i8),
    PushI16(i16),
    PushI32(i32),
    PushI64(i64),
    PushIsize(isize),
    PushU8(u8),
    PushU16(u16),
    PushU32(u32),
    PushU64(u64),
    PushUsize(usize),
    PushF16(f32),
    PushF32(f32),
    PushF64(f64),

    Pop, // Discard top of stack

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Eq
    Eq,
    Lt,
    Gt,
    Leq,
    Geq,
    Not, // Boolean negation
    Neg, // Arithmetic negation

    // Variables
    // In a production VM, strings would be resolved to integer indices at compile time.
    Load(String),
    Store(String),

    // Removes the most recent binding of a variable
    PopBinding(String),

    // Control Flow
    Jump(usize),        // Unconditional jump to index
    JumpIfFalse(usize), // Jumps if the top of the stack is false

    // Creates a closure from the given function address and environment
    MakeClosure {
        addr: usize,
        param: String,
        captures: Vec<String>, // Optimized capture list
    },

    // Recursive closures need a reference to themselves in their environment
    MakeRecursiveClosure {
        addr: usize,
        param: String,
        rec_name: String,
        captures: Vec<String>, // Optimized capture list
    },

    // Structs
    MakeStruct {
        name: String,
        fields: Vec<String>, // Field names corresponding to values on stack
    },
    GetField(String),

    // Calls the function/closure at the top of the stack
    Call,
    // Returns from the current function
    Return,

    // Assert: Pops a boolean. Panics if false.
    Assert,

    // Stop the VM
    Halt,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OpCode::*;
        const CYAN: &str = "\x1b[36m";
        const YELLOW: &str = "\x1b[33m";
        const RESET: &str = "\x1b[0m";

        // Longest opcode mnemonic length (without colors)
        const WIDTH: usize = 14;

        // Formats the mnemonic padded, then wraps in color
        fn op(name: &str) -> String {
            format!("{CYAN}{:<WIDTH$}{RESET}", name)
        }

        match self {
            PushInt(n) => write!(f, "{} {YELLOW}{n}{RESET}", op("PUSH_INT")),
            PushBool(b) => write!(f, "{} {YELLOW}{b}{RESET}", op("PUSH_BOOL")),
            PushI8(n) => write!(f, "{} {YELLOW}{n}i8{RESET}", op("PUSH_I8")),
            PushI16(n) => write!(f, "{} {YELLOW}{n}i16{RESET}", op("PUSH_I16")),
            PushI32(n) => write!(f, "{} {YELLOW}{n}i32{RESET}", op("PUSH_I32")),
            PushI64(n) => write!(f, "{} {YELLOW}{n}i64{RESET}", op("PUSH_I64")),
            PushIsize(n) => write!(f, "{} {YELLOW}{n}isize{RESET}", op("PUSH_ISIZE")),
            PushU8(n) => write!(f, "{} {YELLOW}{n}u8{RESET}", op("PUSH_U8")),
            PushU16(n) => write!(f, "{} {YELLOW}{n}u16{RESET}", op("PUSH_U16")),
            PushU32(n) => write!(f, "{} {YELLOW}{n}u32{RESET}", op("PUSH_U32")),
            PushU64(n) => write!(f, "{} {YELLOW}{n}u64{RESET}", op("PUSH_U64")),
            PushUsize(n) => write!(f, "{} {YELLOW}{n}usize{RESET}", op("PUSH_USIZE")),
            PushF16(n) => write!(f, "{} {YELLOW}{n}f16{RESET}", op("PUSH_F16")),
            PushF32(n) => write!(f, "{} {YELLOW}{n}f32{RESET}", op("PUSH_F32")),
            PushF64(n) => write!(f, "{} {YELLOW}{n}f64{RESET}", op("PUSH_F64")),

            Pop => write!(f, "{}", op("POP")),
            Add => write!(f, "{}", op("ADD")),
            Sub => write!(f, "{}", op("SUB")),
            Mul => write!(f, "{}", op("MUL")),
            Div => write!(f, "{}", op("DIV")),
            Load(s) => write!(f, "{} {YELLOW}{s}{RESET}", op("LOAD")),
            Store(s) => write!(f, "{} {YELLOW}{s}{RESET}", op("STORE")),
            PopBinding(s) => write!(f, "{} {YELLOW}{s}{RESET}", op("POP_BINDING")),
            Jump(n) => write!(f, "{} @{YELLOW}{n}{RESET}", op("JUMP")),
            JumpIfFalse(n) => write!(f, "{} @{YELLOW}{n}{RESET}", op("JUMP_IF_FALSE"),),
            MakeClosure { addr, param, .. } => write!(
                f,
                "{} param:{YELLOW}{param}{RESET} @{YELLOW}{addr}{RESET}",
                op("CLOSURE"),
            ),
            MakeRecursiveClosure {
                addr,
                param,
                rec_name,
                ..
            } => write!(
                f,
                "{} rec:{YELLOW}{rec_name}{RESET} param:{YELLOW}{param}{RESET} @{YELLOW}{addr}{RESET}",
                op("REC_CLOSURE"),
            ),
            MakeStruct { name, .. } => write!(f, "{} {YELLOW}{}{RESET}", op("MAKE_STRUCT"), name),
            GetField(s) => write!(f, "{} {YELLOW}{}{RESET}", op("GET_FIELD"), s),
            Call => write!(f, "{}", op("CALL")),
            Return => write!(f, "{}", op("RETURN")),
            Assert => write!(f, "{}", op("ASSERT")),
            Halt => write!(f, "{}", op("HALT")),

            Eq => write!(f, "{}", op("Eq")),
            Lt => write!(f, "{}", op("Lt")),
            Gt => write!(f, "{}", op("Gt")),
            Leq => write!(f, "{}", op("Leq")),
            Geq => write!(f, "{}", op("Geq")),
            Not => write!(f, "{}", op("Not")),
            Neg => write!(f, "{}", op("Neg")),
        }
    }
}

struct CallFrame {
    return_ip: usize, // Instruction Pointer to return to
    locals: HashMap<String, Vec<Value>>,
}

pub struct VM {
    code: Vec<OpCode>,
    ip: usize,                       // Instruction Pointer
    stack: Vec<Value>,               // Operand Stack
    frames: Vec<CallFrame>,          // Call Stack
    globals: HashMap<String, Value>, // Global scope (optional, or treated as bottom frame)
}

impl VM {
    pub fn new(code: Vec<OpCode>) -> Self {
        // Create a Root Frame so that top-level variables
        // use the same shadowing logic as function variables.
        let root_frame = CallFrame {
            return_ip: 0, // Should not be used since we halt before returning
            locals: HashMap::new(),
        };

        Self {
            code,
            ip: 0,
            stack: Vec::new(),
            frames: vec![root_frame], // <--- FIX: Start with one frame
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<Value, String> {
        // Add a HALT at the end if not present to ensure clean exit
        if !matches!(self.code.last(), Some(OpCode::Halt)) {
            self.code.push(OpCode::Halt);
        }

        loop {
            if self.ip >= self.code.len() {
                return Err("Instruction pointer out of bounds".to_string());
            }

            let op = self.code[self.ip].clone();
            self.ip += 1;

            match op {
                OpCode::Halt => break,

                OpCode::PushInt(n) => self.stack.push(Value::Int(n)),
                OpCode::PushBool(b) => self.stack.push(Value::Bool(b)),
                OpCode::PushI8(n) => self.stack.push(Value::I8(n)),
                OpCode::PushI16(n) => self.stack.push(Value::I16(n)),
                OpCode::PushI32(n) => self.stack.push(Value::I32(n)),
                OpCode::PushI64(n) => self.stack.push(Value::I64(n)),
                OpCode::PushIsize(n) => self.stack.push(Value::Isize(n)),
                OpCode::PushU8(n) => self.stack.push(Value::U8(n)),
                OpCode::PushU16(n) => self.stack.push(Value::U16(n)),
                OpCode::PushU32(n) => self.stack.push(Value::U32(n)),
                OpCode::PushU64(n) => self.stack.push(Value::U64(n)),
                OpCode::PushUsize(n) => self.stack.push(Value::Usize(n)),
                OpCode::PushF16(n) => self.stack.push(Value::F16(n)),
                OpCode::PushF32(n) => self.stack.push(Value::F32(n)),
                OpCode::PushF64(n) => self.stack.push(Value::F64(n)),
                OpCode::Neg => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    let res = match val {
                        Value::Int(n) => Value::Int(-n),
                        Value::I8(n) => Value::I8(-n),
                        Value::I16(n) => Value::I16(-n),
                        Value::I32(n) => Value::I32(-n),
                        Value::I64(n) => Value::I64(-n),
                        Value::Isize(n) => Value::Isize(-n),
                        Value::F16(n) => Value::F16(-n),
                        Value::F32(n) => Value::F32(-n),
                        Value::F64(n) => Value::F64(-n),
                        _ => return Err(format!("Negation not supported for {}", val)),
                    };
                    self.stack.push(res);
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    let rhs = self.stack.pop().ok_or("Stack underflow")?;
                    let lhs = self.stack.pop().ok_or("Stack underflow")?;
                    let res = self.binary_op(lhs, rhs, &op)?;
                    self.stack.push(res);
                }
                OpCode::Eq | OpCode::Lt | OpCode::Gt | OpCode::Leq | OpCode::Geq => {
                    let rhs = self.stack.pop().ok_or("Stack underflow")?;
                    let lhs = self.stack.pop().ok_or("Stack underflow")?;

                    let res = match (&lhs, &rhs, &op) {
                        // Equality
                        (Value::Int(a), Value::Int(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::Bool(a), Value::Bool(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::I8(a), Value::I8(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::I16(a), Value::I16(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::I32(a), Value::I32(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::I64(a), Value::I64(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::Isize(a), Value::Isize(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::U8(a), Value::U8(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::U16(a), Value::U16(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::U32(a), Value::U32(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::U64(a), Value::U64(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::Usize(a), Value::Usize(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::F16(a), Value::F16(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::F32(a), Value::F32(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::F64(a), Value::F64(b), OpCode::Eq) => Value::Bool(a == b),

                        // Less Than
                        (Value::Int(a), Value::Int(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::I8(a), Value::I8(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::I16(a), Value::I16(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::I32(a), Value::I32(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::I64(a), Value::I64(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::Isize(a), Value::Isize(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::U8(a), Value::U8(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::U16(a), Value::U16(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::U32(a), Value::U32(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::U64(a), Value::U64(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::Usize(a), Value::Usize(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::F16(a), Value::F16(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::F32(a), Value::F32(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::F64(a), Value::F64(b), OpCode::Lt) => Value::Bool(a < b),

                        // Greater Than
                        (Value::Int(a), Value::Int(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::I8(a), Value::I8(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::I16(a), Value::I16(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::I32(a), Value::I32(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::I64(a), Value::I64(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::Isize(a), Value::Isize(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::U8(a), Value::U8(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::U16(a), Value::U16(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::U32(a), Value::U32(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::U64(a), Value::U64(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::Usize(a), Value::Usize(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::F16(a), Value::F16(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::F32(a), Value::F32(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::F64(a), Value::F64(b), OpCode::Gt) => Value::Bool(a > b),

                        // Less Than Equals
                        (Value::Int(a), Value::Int(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::I8(a), Value::I8(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::I16(a), Value::I16(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::I32(a), Value::I32(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::I64(a), Value::I64(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::Isize(a), Value::Isize(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::U8(a), Value::U8(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::U16(a), Value::U16(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::U32(a), Value::U32(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::U64(a), Value::U64(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::Usize(a), Value::Usize(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::F16(a), Value::F16(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::F32(a), Value::F32(b), OpCode::Leq) => Value::Bool(a <= b),
                        (Value::F64(a), Value::F64(b), OpCode::Leq) => Value::Bool(a <= b),

                        // Greater Than Equals
                        (Value::Int(a), Value::Int(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::I8(a), Value::I8(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::I16(a), Value::I16(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::I32(a), Value::I32(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::I64(a), Value::I64(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::Isize(a), Value::Isize(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::U8(a), Value::U8(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::U16(a), Value::U16(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::U32(a), Value::U32(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::U64(a), Value::U64(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::Usize(a), Value::Usize(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::F16(a), Value::F16(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::F32(a), Value::F32(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::F64(a), Value::F64(b), OpCode::Geq) => Value::Bool(a >= b),

                        _ => {
                            return Err(format!(
                                "Comparison mismatch for {:?} {:?} {:?}",
                                lhs, op, rhs
                            ));
                        }
                    };
                    self.stack.push(res);
                }
                OpCode::Load(name) => {
                    // Peek the LAST value in the vector (most recent shadowing)
                    let val = if let Some(frame) = self.frames.last() {
                        frame
                            .locals
                            .get(&name)
                            .and_then(|vec| vec.last())
                            .or_else(|| self.globals.get(&name))
                    } else {
                        self.globals.get(&name)
                    };

                    match val {
                        Some(v) => self.stack.push(v.clone()),
                        None => return Err(format!("Undefined variable '{}'", name)),
                    }
                }

                OpCode::Store(name) => {
                    // POP the value. It moves from Operand Stack -> Environment
                    let val = self.stack.pop().ok_or("Stack underflow")?;

                    if let Some(frame) = self.frames.last_mut() {
                        // Push to the vector for this variable (Shadowing)
                        frame.locals.entry(name).or_insert_with(Vec::new).push(val);
                    } else {
                        // Global scope (optional fallback)
                        self.globals.insert(name, val);
                    }
                }

                OpCode::Jump(addr) => {
                    self.ip = addr;
                }

                OpCode::JumpIfFalse(addr) => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(false) = val {
                        self.ip = addr;
                    } else if !matches!(val, Value::Bool(_)) {
                        return Err("Condition must be a boolean".to_string());
                    }
                }

                OpCode::MakeClosure {
                    addr,
                    param,
                    captures,
                } => {
                    let mut flat_env = HashMap::new();
                    if let Some(frame) = self.frames.last() {
                        for cap_name in captures {
                            if let Some(vals) = frame.locals.get(&cap_name) {
                                if let Some(val) = vals.last() {
                                    flat_env.insert(cap_name.clone(), val.clone());
                                }
                            }
                            // If not in locals, could be global, but globals are accessible anyway.
                            // We only capture locals.
                        }
                    }

                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                        rec_name: None,
                    });
                }
                OpCode::MakeRecursiveClosure {
                    addr,
                    param,
                    rec_name,
                    captures,
                } => {
                    let mut flat_env = HashMap::new();
                    if let Some(frame) = self.frames.last() {
                        for cap_name in captures {
                            if let Some(vals) = frame.locals.get(&cap_name) {
                                if let Some(val) = vals.last() {
                                    flat_env.insert(cap_name.clone(), val.clone());
                                }
                            }
                        }
                    }

                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                        rec_name: Some(rec_name),
                    });
                }

                OpCode::Call => {
                    let arg = self.stack.pop().ok_or("Stack underflow (arg)")?;
                    let func = self.stack.pop().ok_or("Stack underflow (func)")?;

                    if let Value::Closure {
                        addr,
                        param,
                        env,
                        rec_name,
                    } = &func
                    {
                        let mut local_env: HashMap<String, Vec<Value>> = HashMap::new();
                        for (k, v) in env {
                            local_env.insert(k.clone(), vec![v.clone()]);
                        }
                        local_env.entry(param.clone()).or_default().push(arg);

                        if let Some(rname) = rec_name {
                            local_env
                                .entry(rname.clone())
                                .or_default()
                                .push(func.clone());
                        }

                        self.frames.push(CallFrame {
                            return_ip: self.ip,
                            locals: local_env,
                        });
                        self.ip = *addr;
                    } else {
                        return Err("Trying to call a non-function value".to_string());
                    }
                }

                OpCode::MakeStruct { name, fields } => {
                    let mut struct_val_fields = HashMap::new();
                    // Fields are pushed in order, so popping gives reverse order
                    // field_names in opcode is [f1, f2, f3]
                    // Stack is [v1, v2, v3] (top)
                    // Pop -> v3 (matches f3)
                    for field_name in fields.iter().rev() {
                        let val = self.stack.pop().ok_or("Stack underflow for struct init")?;
                        struct_val_fields.insert(field_name.clone(), val);
                    }
                    self.stack.push(Value::Struct {
                        name: name.clone(),
                        fields: struct_val_fields,
                    });
                }

                OpCode::GetField(field_name) => {
                    let obj = self.stack.pop().ok_or("Stack underflow for field access")?;
                    if let Value::Struct { name: _, fields } = obj {
                        let val = fields
                            .get(&field_name)
                            .ok_or_else(|| format!("Field '{}' not found in struct", field_name))?;
                        self.stack.push(val.clone());
                    } else {
                        return Err(format!("Field access on non-struct value: {}", obj));
                    }
                }

                OpCode::Return => {
                    if let Some(frame) = self.frames.pop() {
                        self.ip = frame.return_ip;
                    } else {
                        // Returned from top level? Just stop.
                        break;
                    }
                }
                OpCode::PopBinding(name) => {
                    if let Some(frame) = self.frames.last_mut()
                        && let Some(vec) = frame.locals.get_mut(&name)
                    {
                        vec.pop();
                        // Optional: clean up empty vectors
                        if vec.is_empty() {
                            frame.locals.remove(&name);
                        }
                    }
                    // We don't pop globals in this language
                }

                OpCode::Assert => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    match val {
                        Value::Bool(true) => {
                            self.stack.push(Value::Bool(true));
                        }
                        Value::Bool(false) => {
                            return Err(format!("Assertion failed at IP:{}", self.ip - 1));
                        }
                        _ => return Err("Assert requires boolean".to_string()),
                    }
                }

                OpCode::Not => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                    } else {
                        return Err(format!("'not' requires bool, got {}", val));
                    }
                }
            }
        }

        self.stack
            .pop()
            .ok_or("Stack empty after execution".to_string())
    }

    fn binary_op(&self, lhs: Value, rhs: Value, op: &OpCode) -> Result<Value, String> {
        let res = match (lhs, rhs, op) {
            // BigInt
            (Value::Int(a), Value::Int(b), OpCode::Add) => Value::Int(a + b),
            (Value::Int(a), Value::Int(b), OpCode::Sub) => Value::Int(a - b),
            (Value::Int(a), Value::Int(b), OpCode::Mul) => Value::Int(a * b),
            (Value::Int(a), Value::Int(b), OpCode::Div) => {
                let zero = BigInt::from(0);
                if b == zero {
                    return Err("Division by zero".to_string());
                }
                Value::Int(a / b)
            }
            // Strict Primitive Arithmetic (Macros would be cleaner, but expanding for clarity)
            // I8
            (Value::I8(a), Value::I8(b), OpCode::Add) => Value::I8(a.wrapping_add(b)),
            (Value::I8(a), Value::I8(b), OpCode::Sub) => Value::I8(a.wrapping_sub(b)),
            (Value::I8(a), Value::I8(b), OpCode::Mul) => Value::I8(a.wrapping_mul(b)),
            (Value::I8(a), Value::I8(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::I8(a.wrapping_div(b))
            }
            // I16
            (Value::I16(a), Value::I16(b), OpCode::Add) => Value::I16(a.wrapping_add(b)),
            (Value::I16(a), Value::I16(b), OpCode::Sub) => Value::I16(a.wrapping_sub(b)),
            (Value::I16(a), Value::I16(b), OpCode::Mul) => Value::I16(a.wrapping_mul(b)),
            (Value::I16(a), Value::I16(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::I16(a.wrapping_div(b))
            }
            // I32
            (Value::I32(a), Value::I32(b), OpCode::Add) => Value::I32(a.wrapping_add(b)),
            (Value::I32(a), Value::I32(b), OpCode::Sub) => Value::I32(a.wrapping_sub(b)),
            (Value::I32(a), Value::I32(b), OpCode::Mul) => Value::I32(a.wrapping_mul(b)),
            (Value::I32(a), Value::I32(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::I32(a.wrapping_div(b))
            }
            // I64
            (Value::I64(a), Value::I64(b), OpCode::Add) => Value::I64(a.wrapping_add(b)),
            (Value::I64(a), Value::I64(b), OpCode::Sub) => Value::I64(a.wrapping_sub(b)),
            (Value::I64(a), Value::I64(b), OpCode::Mul) => Value::I64(a.wrapping_mul(b)),
            (Value::I64(a), Value::I64(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::I64(a.wrapping_div(b))
            }
            // Isize
            (Value::Isize(a), Value::Isize(b), OpCode::Add) => Value::Isize(a.wrapping_add(b)),
            (Value::Isize(a), Value::Isize(b), OpCode::Sub) => Value::Isize(a.wrapping_sub(b)),
            (Value::Isize(a), Value::Isize(b), OpCode::Mul) => Value::Isize(a.wrapping_mul(b)),
            (Value::Isize(a), Value::Isize(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::Isize(a.wrapping_div(b))
            }

            // U8
            (Value::U8(a), Value::U8(b), OpCode::Add) => Value::U8(a.wrapping_add(b)),
            (Value::U8(a), Value::U8(b), OpCode::Sub) => Value::U8(a.wrapping_sub(b)),
            (Value::U8(a), Value::U8(b), OpCode::Mul) => Value::U8(a.wrapping_mul(b)),
            (Value::U8(a), Value::U8(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::U8(a.wrapping_div(b))
            }
            // U16
            (Value::U16(a), Value::U16(b), OpCode::Add) => Value::U16(a.wrapping_add(b)),
            (Value::U16(a), Value::U16(b), OpCode::Sub) => Value::U16(a.wrapping_sub(b)),
            (Value::U16(a), Value::U16(b), OpCode::Mul) => Value::U16(a.wrapping_mul(b)),
            (Value::U16(a), Value::U16(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::U16(a.wrapping_div(b))
            }
            // U32
            (Value::U32(a), Value::U32(b), OpCode::Add) => Value::U32(a.wrapping_add(b)),
            (Value::U32(a), Value::U32(b), OpCode::Sub) => Value::U32(a.wrapping_sub(b)),
            (Value::U32(a), Value::U32(b), OpCode::Mul) => Value::U32(a.wrapping_mul(b)),
            (Value::U32(a), Value::U32(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::U32(a.wrapping_div(b))
            }
            // U64
            (Value::U64(a), Value::U64(b), OpCode::Add) => Value::U64(a.wrapping_add(b)),
            (Value::U64(a), Value::U64(b), OpCode::Sub) => Value::U64(a.wrapping_sub(b)),
            (Value::U64(a), Value::U64(b), OpCode::Mul) => Value::U64(a.wrapping_mul(b)),
            (Value::U64(a), Value::U64(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::U64(a.wrapping_div(b))
            }
            // Usize
            (Value::Usize(a), Value::Usize(b), OpCode::Add) => Value::Usize(a.wrapping_add(b)),
            (Value::Usize(a), Value::Usize(b), OpCode::Sub) => Value::Usize(a.wrapping_sub(b)),
            (Value::Usize(a), Value::Usize(b), OpCode::Mul) => Value::Usize(a.wrapping_mul(b)),
            (Value::Usize(a), Value::Usize(b), OpCode::Div) => {
                if b == 0 {
                    return Err("Div by zero".into());
                }
                Value::Usize(a.wrapping_div(b))
            }

            // Floats (F16 stored as F32)
            (Value::F16(a), Value::F16(b), OpCode::Add) => Value::F16(a + b),
            (Value::F16(a), Value::F16(b), OpCode::Sub) => Value::F16(a - b),
            (Value::F16(a), Value::F16(b), OpCode::Mul) => Value::F16(a * b),
            (Value::F16(a), Value::F16(b), OpCode::Div) => Value::F16(a / b),
            // F32
            (Value::F32(a), Value::F32(b), OpCode::Add) => Value::F32(a + b),
            (Value::F32(a), Value::F32(b), OpCode::Sub) => Value::F32(a - b),
            (Value::F32(a), Value::F32(b), OpCode::Mul) => Value::F32(a * b),
            (Value::F32(a), Value::F32(b), OpCode::Div) => Value::F32(a / b),
            // F64
            (Value::F64(a), Value::F64(b), OpCode::Add) => Value::F64(a + b),
            (Value::F64(a), Value::F64(b), OpCode::Sub) => Value::F64(a - b),
            (Value::F64(a), Value::F64(b), OpCode::Mul) => Value::F64(a * b),
            (Value::F64(a), Value::F64(b), OpCode::Div) => Value::F64(a / b),

            _ => return Err("Type mismatch or invalid operator".to_string()),
        };
        Ok(res)
    }

    pub fn debug_run(&mut self) -> Result<Value, String> {
        // Ensure we halt at the end
        if !matches!(self.code.last(), Some(OpCode::Halt)) {
            self.code.push(OpCode::Halt);
        }

        println!("=== START DEBUG TRACE ===");
        loop {
            if self.ip >= self.code.len() {
                return Err("Instruction pointer out of bounds".to_string());
            }

            let op = self.code[self.ip].clone();

            // --- LOGGING ---
            print!(
                "[IP:{:03}] {:<25} | Stack: {:?}",
                self.ip,
                op.to_string(),
                self.stack
            );
            if let Some(frame) = self.frames.last() {
                // Print a simplified view of locals (last value of each var)
                print!(" | Locals: {{ ");
                for (k, v) in &frame.locals {
                    if let Some(val) = v.last() {
                        print!("{}:{} ", k, val);
                    }
                }
                print!("}}");
            }
            println!();
            // ----------------

            self.ip += 1; // Advance IP

            match op {
                OpCode::MakeStruct { name, fields } => {
                    let mut struct_val_fields = HashMap::new();
                    // Same logic as run
                    for field_name in fields.iter().rev() {
                        let val = self.stack.pop().ok_or("Stack underflow")?;
                        struct_val_fields.insert(field_name.clone(), val);
                    }
                    self.stack.push(Value::Struct {
                        name: name.clone(),
                        fields: struct_val_fields,
                    });
                }
                OpCode::GetField(field_name) => {
                    let obj = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Struct { fields, .. } = obj {
                        let val = fields.get(&field_name).ok_or("Field not found")?;
                        self.stack.push(val.clone());
                    } else {
                        return Err("Not a struct".to_string());
                    }
                }
                OpCode::Halt => break,

                OpCode::PushInt(n) => self.stack.push(Value::Int(n)),
                OpCode::PushBool(b) => self.stack.push(Value::Bool(b)),
                OpCode::PushI8(n) => self.stack.push(Value::I8(n)),
                OpCode::PushI16(n) => self.stack.push(Value::I16(n)),
                OpCode::PushI32(n) => self.stack.push(Value::I32(n)),
                OpCode::PushI64(n) => self.stack.push(Value::I64(n)),
                OpCode::PushIsize(n) => self.stack.push(Value::Isize(n)),
                OpCode::PushU8(n) => self.stack.push(Value::U8(n)),
                OpCode::PushU16(n) => self.stack.push(Value::U16(n)),
                OpCode::PushU32(n) => self.stack.push(Value::U32(n)),
                OpCode::PushU64(n) => self.stack.push(Value::U64(n)),
                OpCode::PushUsize(n) => self.stack.push(Value::Usize(n)),
                OpCode::PushF16(n) => self.stack.push(Value::F16(n)),
                OpCode::PushF32(n) => self.stack.push(Value::F32(n)),
                OpCode::PushF64(n) => self.stack.push(Value::F64(n)),

                OpCode::Neg => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    let res = match val {
                        Value::Int(n) => Value::Int(-n),
                        Value::I8(n) => Value::I8(-n),
                        Value::I16(n) => Value::I16(-n),
                        Value::I32(n) => Value::I32(-n),
                        Value::I64(n) => Value::I64(-n),
                        Value::Isize(n) => Value::Isize(-n),
                        // Floats
                        Value::F16(n) => Value::F16(-n),
                        Value::F32(n) => Value::F32(-n),
                        Value::F64(n) => Value::F64(-n),
                        _ => return Err(format!("Negation not supported for {}", val)),
                    };
                    self.stack.push(res);
                }

                OpCode::Pop => {
                    self.stack.pop();
                }

                // Arithmetic (Use binary_op)
                OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let res = self.binary_op(a, b, &op)?;
                    self.stack.push(res);
                }

                // Logic (Use binary_op later or keep inline if specific)
                OpCode::Not => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(b) = val {
                        self.stack.push(Value::Bool(!b));
                    } else {
                        return Err("Not requires bool".to_string());
                    }
                }

                OpCode::Eq | OpCode::Lt | OpCode::Gt | OpCode::Leq | OpCode::Geq => {
                    let rhs = self.stack.pop().ok_or("Stack underflow")?;
                    let lhs = self.stack.pop().ok_or("Stack underflow")?;

                    let res = match (&lhs, &rhs, &op) {
                        (Value::Int(a), Value::Int(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::Bool(a), Value::Bool(b), OpCode::Eq) => Value::Bool(a == b),
                        // Primitives Eq
                        (Value::I32(a), Value::I32(b), OpCode::Eq) => Value::Bool(a == b),
                        (Value::F64(a), Value::F64(b), OpCode::Eq) => Value::Bool(a == b),
                        // TODO: All primitives
                        (Value::Int(a), Value::Int(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::Int(a), Value::Int(b), OpCode::Geq) => Value::Bool(a >= b),
                        (Value::Int(a), Value::Int(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::Int(a), Value::Int(b), OpCode::Leq) => Value::Bool(a <= b),

                        _ => {
                            return Err(format!(
                                "Comparison mismatch or unsupported types: {:?} {:?}",
                                lhs, rhs
                            ));
                        }
                    };
                    self.stack.push(res);
                }

                // Variable Access
                OpCode::Load(name) => {
                    let val = if let Some(frame) = self.frames.last() {
                        frame
                            .locals
                            .get(&name)
                            .and_then(|v| v.last())
                            .or_else(|| self.globals.get(&name))
                    } else {
                        self.globals.get(&name)
                    };

                    match val {
                        Some(v) => self.stack.push(v.clone()),
                        None => return Err(format!("Undefined variable '{}'", name)),
                    }
                }

                OpCode::Store(name) => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Some(frame) = self.frames.last_mut() {
                        frame.locals.entry(name).or_insert_with(Vec::new).push(val);
                    } else {
                        // Should not happen if VM::new initializes a root frame,
                        // but safe fallback:
                        self.globals.insert(name, val);
                    }
                }

                OpCode::PopBinding(name) => {
                    if let Some(frame) = self.frames.last_mut()
                        && let Some(vec) = frame.locals.get_mut(&name)
                    {
                        vec.pop();
                        if vec.is_empty() {
                            frame.locals.remove(&name);
                        }
                    }
                }

                // Jumps
                OpCode::Jump(addr) => {
                    self.ip = addr;
                }
                OpCode::JumpIfFalse(addr) => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::Bool(false) = val {
                        self.ip = addr;
                    }
                }

                // Functions
                OpCode::MakeClosure {
                    addr,
                    param,
                    captures,
                } => {
                    let mut flat_env = HashMap::new();
                    // Capture from current frame
                    if let Some(frame) = self.frames.last() {
                        for cap_name in captures {
                            if let Some(vals) = frame.locals.get(&cap_name) {
                                if let Some(val) = vals.last() {
                                    flat_env.insert(cap_name.clone(), val.clone());
                                }
                            }
                        }
                    }
                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                        rec_name: None,
                    });
                }
                OpCode::MakeRecursiveClosure {
                    addr,
                    param,
                    rec_name,
                    captures,
                } => {
                    let mut flat_env = HashMap::new();
                    if let Some(frame) = self.frames.last() {
                        for cap_name in captures {
                            if let Some(vals) = frame.locals.get(&cap_name) {
                                if let Some(val) = vals.last() {
                                    flat_env.insert(cap_name.clone(), val.clone());
                                }
                            }
                        }
                    }
                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                        rec_name: Some(rec_name),
                    });
                }
                OpCode::Call => {
                    let arg = self.stack.pop().ok_or("Stack underflow (arg)")?;
                    let func = self.stack.pop().ok_or("Stack underflow (func)")?;

                    if let Value::Closure {
                        addr,
                        param,
                        env,
                        rec_name,
                    } = &func
                    {
                        // Convert flat env back to shadowing-capable env
                        let mut local_env: HashMap<String, Vec<Value>> = HashMap::new();
                        for (k, v) in env {
                            local_env.insert(k.clone(), vec![v.clone()]);
                        }
                        // Bind argument
                        local_env.entry(param.clone()).or_default().push(arg);

                        if let Some(rname) = rec_name {
                            local_env
                                .entry(rname.clone())
                                .or_default()
                                .push(func.clone());
                        }

                        self.frames.push(CallFrame {
                            return_ip: self.ip,
                            locals: local_env,
                        });
                        self.ip = *addr;
                    } else {
                        return Err("Trying to call a non-function value".to_string());
                    }
                }
                OpCode::Return => {
                    if let Some(frame) = self.frames.pop() {
                        self.ip = frame.return_ip;
                    } else {
                        break;
                    }
                }
                OpCode::Assert => {
                    let val = self.stack.pop().ok_or("Stack underflow")?;
                    match val {
                        Value::Bool(true) => {
                            // Pass
                            // Should we push true back?
                            // If `assert x` is an expression, it needs a value.
                            // In Checker we return Bool. So let's push true.
                            self.stack.push(Value::Bool(true));
                        }
                        Value::Bool(false) => {
                            return Err(format!("Assertion failed at IP:{}", self.ip - 1));
                        }
                        _ => return Err("Assert requires boolean".to_string()),
                    }
                }
            }
        }

        println!("=== END TRACE ===");
        self.stack.pop().ok_or("Stack empty after execution".into())
    }
}
