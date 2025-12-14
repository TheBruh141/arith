use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    // A Closure contains:
    // 1. The instruction index where the function code starts.
    // 2. The name of the argument variable (so we can bind it when called).
    // 3. The Captured Environment (variables from outer scopes).
    Closure {
        addr: usize,
        param: String,
        env: HashMap<String, Value>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Closure { .. } => write!(f, "<function>"),
        }
    }
}

// --- Bytecode Instructions ---

#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack manipulation
    PushInt(i64),
    PushBool(bool),
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

    // Variables
    // In a production VM, strings would be resolved to integer indices at compile time.
    Load(String),
    Store(String),

    // Removes the most recent binding of a variable
    PopBinding(String),

    // Control Flow
    Jump(usize),        // Unconditional jump to index
    JumpIfFalse(usize), // Jump if top of stack is false

    // Functions
    // Creates a closure from the code at `addr`, expecting parameter `param_name`
    MakeClosure { addr: usize, param: String },
    // Calls the function/closure at the top of the stack
    Call,
    // Returns from the current function
    Return,

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
            PushInt(n) => write!(f, "{} {}", op("PUSH_INT"), format!("{YELLOW}{n}{RESET}")),
            PushBool(b) => write!(f, "{} {}", op("PUSH_BOOL"), format!("{YELLOW}{b}{RESET}")),
            Pop => write!(f, "{}", op("POP")),
            Add => write!(f, "{}", op("ADD")),
            Sub => write!(f, "{}", op("SUB")),
            Mul => write!(f, "{}", op("MUL")),
            Div => write!(f, "{}", op("DIV")),
            Load(s) => write!(f, "{} {}", op("LOAD"), format!("{YELLOW}{s}{RESET}")),
            Store(s) => write!(f, "{} {}", op("STORE"), format!("{YELLOW}{s}{RESET}")),
            PopBinding(s) => write!(f, "{} {}", op("POP_BINDING"), format!("{YELLOW}{s}{RESET}")),
            Jump(n) => write!(f, "{} @{}", op("JUMP"), format!("{YELLOW}{n}{RESET}")),
            JumpIfFalse(n) => write!(
                f,
                "{} @{}",
                op("JUMP_IF_FALSE"),
                format!("{YELLOW}{n}{RESET}")
            ),
            MakeClosure { addr, param } => write!(
                f,
                "{} param:{} @{}",
                op("CLOSURE"),
                format!("{YELLOW}{param}{RESET}"),
                format!("{YELLOW}{addr}{RESET}"),
            ),
            Call => write!(f, "{}", op("CALL")),
            Return => write!(f, "{}", op("RETURN")),
            Halt => write!(f, "{}", op("HALT")),

            Eq => write!(f, "{}", op("Eq")),
            Lt => write!(f, "{}", op("Lt")),
            Gt => write!(f, "{}", op("Gt")),
            Leq => write!(f, "{}", op("Leq")),
            Geq => write!(f, "{}", op("Geq")),
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

                    let res = match (lhs, rhs, &op) {
                        (Value::Int(a), Value::Int(b), OpCode::Eq) => Value::Bool(a == b),

                        (Value::Int(a), Value::Int(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::Int(a), Value::Int(b), OpCode::Geq) => Value::Bool(a >= b),

                        (Value::Int(a), Value::Int(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::Int(a), Value::Int(b), OpCode::Leq) => Value::Bool(a <= b),

                        _ => return Err("Comparison requires Integers".to_string()),
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

                OpCode::MakeClosure { addr, param } => {
                    // Flatten the current environment for the closure
                    // (Take the top of every stack)
                    let mut flat_env = HashMap::new();
                    let current_locals = if let Some(frame) = self.frames.last() {
                        &frame.locals
                    } else {
                        // Empty map if no frame (shouldn't happen in this logic but safe to handle)
                        return Err("Closure created outside frame context".into());
                    };

                    for (k, v_vec) in current_locals {
                        if let Some(top_val) = v_vec.last() {
                            flat_env.insert(k.clone(), top_val.clone());
                        }
                    }

                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                    });
                }

                OpCode::Call => {
                    let arg = self.stack.pop().ok_or("Stack underflow (arg)")?;
                    let func = self.stack.pop().ok_or("Stack underflow (func)")?;

                    if let Value::Closure {
                        addr,
                        param,
                        env,
                    } = func
                    {
                        // Convert flat env back to shadowing-capable env
                        let mut local_env: HashMap<String, Vec<Value>> = HashMap::new();
                        for (k, v) in env {
                            local_env.insert(k, vec![v]);
                        }
                        // Bind argument
                        local_env.entry(param).or_default().push(arg);

                        self.frames.push(CallFrame {
                            return_ip: self.ip,
                            locals: local_env,
                        });
                        self.ip = addr;
                    } else {
                        return Err("Trying to call a non-function value".to_string());
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
                        && let Some(vec) = frame.locals.get_mut(&name) {
                            vec.pop();
                            // Optional: clean up empty vectors
                            if vec.is_empty() {
                                frame.locals.remove(&name);
                            }
                        }
                    // We don't pop globals in this language
                }
            }
        }

        self.stack
            .pop()
            .ok_or("Stack empty after execution".to_string())
    }

    fn binary_op(&self, lhs: Value, rhs: Value, op: &OpCode) -> Result<Value, String> {
        match (lhs, rhs, op) {
            (Value::Int(a), Value::Int(b), OpCode::Add) => Ok(Value::Int(a + b)),
            (Value::Int(a), Value::Int(b), OpCode::Sub) => Ok(Value::Int(a - b)),
            (Value::Int(a), Value::Int(b), OpCode::Mul) => Ok(Value::Int(a * b)),
            (Value::Int(a), Value::Int(b), OpCode::Div) => {
                if b == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            _ => Err("Type mismatch or invalid operator".to_string()),
        }
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

            self.ip += 1;

            match op {
                OpCode::Halt => break,

                OpCode::PushInt(n) => self.stack.push(Value::Int(n)),
                OpCode::PushBool(b) => self.stack.push(Value::Bool(b)),

                OpCode::Pop => {
                    self.stack.pop();
                }

                // Arithmetic
                OpCode::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                        _ => return Err("Type mismatch Add".into()),
                    }
                }
                OpCode::Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x - y)),
                        _ => return Err("Type mismatch Sub".into()),
                    }
                }
                OpCode::Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x * y)),
                        _ => return Err("Type mismatch Mul".into()),
                    }
                }
                OpCode::Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            if y == 0 {
                                return Err("Div by zero".into());
                            }
                            self.stack.push(Value::Int(x / y));
                        }
                        _ => return Err("Type mismatch Div".into()),
                    }
                }
                OpCode::Eq | OpCode::Lt | OpCode::Gt | OpCode::Leq | OpCode::Geq => {
                    let rhs = self.stack.pop().ok_or("Stack underflow")?;
                    let lhs = self.stack.pop().ok_or("Stack underflow")?;

                    let res = match (lhs, rhs, &op) {
                        (Value::Int(a), Value::Int(b), OpCode::Eq) => Value::Bool(a == b),

                        (Value::Int(a), Value::Int(b), OpCode::Lt) => Value::Bool(a < b),
                        (Value::Int(a), Value::Int(b), OpCode::Geq) => Value::Bool(a >= b),

                        (Value::Int(a), Value::Int(b), OpCode::Gt) => Value::Bool(a > b),
                        (Value::Int(a), Value::Int(b), OpCode::Leq) => Value::Bool(a <= b),

                        _ => return Err("Comparison requires Integers".to_string()),
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
                        && let Some(vec) = frame.locals.get_mut(&name) {
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
                OpCode::MakeClosure { addr, param } => {
                    let mut flat_env = HashMap::new();
                    // Capture from current frame
                    if let Some(frame) = self.frames.last() {
                        for (k, v_vec) in &frame.locals {
                            if let Some(top_val) = v_vec.last() {
                                flat_env.insert(k.clone(), top_val.clone());
                            }
                        }
                    }
                    self.stack.push(Value::Closure {
                        addr,
                        param,
                        env: flat_env,
                    });
                }

                OpCode::Call => {
                    let arg = self.stack.pop().ok_or("Stack underflow")?;
                    let func = self.stack.pop().ok_or("Stack underflow")?;

                    if let Value::Closure { addr, param, env } = func {
                        let mut local_env: HashMap<String, Vec<Value>> = HashMap::new();
                        // Hydrate environment (env map -> shadowing stacks)
                        for (k, v) in env {
                            local_env.insert(k, vec![v]);
                        }
                        // Bind argument
                        local_env.entry(param).or_default().push(arg);

                        self.frames.push(CallFrame {
                            return_ip: self.ip,
                            locals: local_env,
                        });
                        self.ip = addr;
                    } else {
                        return Err("Calling non-function".into());
                    }
                }

                OpCode::Return => {
                    if let Some(frame) = self.frames.pop() {
                        self.ip = frame.return_ip;
                    } else {
                        // Returning from root frame implies end of program
                        break;
                    }
                }
            }
        }

        println!("=== END TRACE ===");
        self.stack.pop().ok_or("Stack empty after execution".into())
    }
}
