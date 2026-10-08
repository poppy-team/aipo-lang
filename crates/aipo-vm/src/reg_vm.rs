//! Virtual register execution engine for 32-bit instructions (Marco 3 / ADP-014).

use crate::arena::ArenaAllocator;
use crate::fault::VmFault;
use crate::value::{DictMap, StructInstance, Value, check_safe_int};
use aipo_bytecode::{Constant, RegCompiledFunction, RegCompiledModule, RegInstruction, RegOpCode};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Maximum nested user-function call depth (bounds Rust stack + RAM).
pub const REG_MAX_CALL_DEPTH: usize = 64;

/// A saved caller activation for nested `RegVm` calls.
#[derive(Debug, Clone)]
pub struct RegCallFrame {
    /// Instruction offset to resume in the caller.
    pub return_pc: usize,
    /// Register receiving the callee result.
    pub return_reg: u8,
}

/// Compact virtual register machine for embedded execution and fast scripts.
#[derive(Debug)]
pub struct RegVm {
    /// 256 virtual registers (fixed array of 16-byte Values = 4 KiB total).
    pub registers: [Value; 256],
    /// Constant pool.
    pub constants: Vec<Value>,
    /// Global variables environment.
    pub globals: HashMap<String, Value>,
    /// Compiled user-function table for `MakeFunction`/`Call`.
    pub functions: Vec<RegCompiledFunction>,
    /// Declared struct definitions: name -> fields.
    pub struct_defs: HashMap<String, Vec<(String, bool)>>,
    /// Active nested-call frames (depth guard).
    pub frames: Vec<RegCallFrame>,
    /// Active exception/failure handler stack: `(handler_pc, err_reg)`.
    pub handler_stack: Vec<(usize, u8)>,
    /// Maximum nested-call depth.
    pub max_call_depth: usize,
    /// Linear bump arena for string/buffer allocations.
    pub arena: ArenaAllocator,
    /// Program counter.
    pub pc: usize,
}

impl Default for RegVm {
    fn default() -> Self {
        Self::new()
    }
}

impl RegVm {
    /// Constructs a new register machine with all registers initialized to `none`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registers: std::array::from_fn(|_| Value::None),
            constants: Vec::new(),
            globals: HashMap::new(),
            functions: Vec::new(),
            struct_defs: HashMap::new(),
            frames: Vec::new(),
            handler_stack: Vec::new(),
            max_call_depth: REG_MAX_CALL_DEPTH,
            arena: ArenaAllocator::with_default_capacity(),
            pc: 0,
        }
    }

    /// Converts a bytecode constant into a runtime value.
    fn convert_constant(c: &Constant) -> Value {
        match c {
            Constant::Nil => Value::None,
            Constant::Bool(b) => Value::Bool(*b),
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(f) => Value::Float(*f),
            Constant::String(s) => Value::String(Rc::new(s.clone())),
        }
    }

    /// Loads a compiled register module (function table for `MakeFunction`).
    pub fn load_module(&mut self, module: &RegCompiledModule) {
        self.functions = module.functions.clone();
        self.struct_defs = module.struct_defs.clone();
    }

    /// Executes a compiled register module's top-level script.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error occurs.
    pub fn run_module(&mut self, module: &RegCompiledModule) -> Result<Value, VmFault> {
        self.functions = module.functions.clone();
        self.struct_defs = module.struct_defs.clone();
        self.frames.clear();
        self.run_function(&module.top_level)
    }

    /// Invokes a compiled user function with isolated registers.
    ///
    /// # Errors
    /// Returns [`VmFault`] on arity mismatch, depth overflow, or runtime faults.
    pub fn invoke_function(
        &mut self,
        func: &RegCompiledFunction,
        args: &[Value],
    ) -> Result<Value, VmFault> {
        if self.frames.len() >= self.max_call_depth {
            return Err(VmFault::CorruptedBytecode {
                offset: self.pc.saturating_sub(1),
                reason: format!(
                    "call stack overflow: max depth {} exceeded",
                    self.max_call_depth
                ),
            });
        }
        self.frames.push(RegCallFrame {
            return_pc: self.pc,
            return_reg: 0,
        });
        let saved_registers =
            std::mem::replace(&mut self.registers, std::array::from_fn(|_| Value::None));
        let saved_constants = std::mem::take(&mut self.constants);
        let saved_pc = self.pc;
        for (i, arg) in args.iter().enumerate().take(256) {
            self.registers[i] = arg.clone();
        }
        for i in args.len()..func.arity.min(256) {
            self.registers[i] = Value::Unset;
        }
        self.constants = func.constants.iter().map(Self::convert_constant).collect();
        let outcome = self.run(&func.instructions);
        self.registers = saved_registers;
        self.constants = saved_constants;
        self.pc = saved_pc;
        self.frames.pop();
        outcome
    }

    /// Executes a compiled register function.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error occurs.
    pub fn run_function(
        &mut self,
        func: &aipo_bytecode::RegCompiledFunction,
    ) -> Result<Value, VmFault> {
        self.constants = func.constants.iter().map(Self::convert_constant).collect();
        self.frames.clear();
        self.run(&func.instructions)
    }

    /// Executes instructions sequentially until `Return` or error.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error (e.g. division by zero, overflow) occurs.
    pub fn run(&mut self, code: &[RegInstruction]) -> Result<Value, VmFault> {
        self.pc = 0;
        while self.pc < code.len() {
            let inst = code[self.pc];
            self.pc += 1;

            let Some(op) = inst.opcode() else {
                return Err(VmFault::CorruptedBytecode {
                    offset: self.pc - 1,
                    reason: format!("unknown opcode {}", inst.raw_opcode()),
                });
            };

            let a = inst.a() as usize;
            let b = inst.b() as usize;
            let c = inst.c() as usize;

            match op {
                RegOpCode::Nop => {}
                RegOpCode::Move => {
                    self.registers[a] = self.registers[b].clone();
                }
                RegOpCode::LoadConst => {
                    let bx = inst.bx() as usize;
                    let val = self.constants.get(bx).cloned().unwrap_or(Value::None);
                    self.registers[a] = val;
                }
                RegOpCode::LoadInt => {
                    self.registers[a] = Value::Int(i64::from(inst.sbx()));
                }
                RegOpCode::LoadNil => {
                    self.registers[a] = Value::None;
                }
                RegOpCode::LoadUnset => {
                    self.registers[a] = Value::Unset;
                }
                RegOpCode::LoadBool => {
                    self.registers[a] = Value::Bool(b != 0);
                }
                RegOpCode::Add => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        let sum = x.checked_add(*y).ok_or(VmFault::Overflow {
                            details: format!("{x} + {y} overflowed safe integer bounds"),
                        })?;
                        self.registers[a] = Value::Int(check_safe_int(sum)?);
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.registers[a] = Value::Float(x + y);
                    }
                    (Value::String(x), Value::String(y)) => {
                        let mut combined = String::with_capacity(x.len() + y.len());
                        combined.push_str(x);
                        combined.push_str(y);
                        self.registers[a] = Value::String(Rc::new(combined));
                    }
                    (v1, v2) => {
                        return Err(VmFault::TypeMismatch {
                            expected: "matching numeric or string types".to_string(),
                            actual: format!("{} and {}", v1.type_name(), v2.type_name()),
                        });
                    }
                },
                RegOpCode::Sub => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        let diff = x.checked_sub(*y).ok_or(VmFault::Overflow {
                            details: format!("{x} - {y} underflowed safe integer bounds"),
                        })?;
                        self.registers[a] = Value::Int(check_safe_int(diff)?);
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.registers[a] = Value::Float(x - y);
                    }
                    (v1, v2) => {
                        return Err(VmFault::TypeMismatch {
                            expected: "numbers".to_string(),
                            actual: format!("{} and {}", v1.type_name(), v2.type_name()),
                        });
                    }
                },
                RegOpCode::Mul => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        let prod = x.checked_mul(*y).ok_or(VmFault::Overflow {
                            details: format!("{x} * {y} overflowed safe integer bounds"),
                        })?;
                        self.registers[a] = Value::Int(check_safe_int(prod)?);
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        self.registers[a] = Value::Float(x * y);
                    }
                    (v1, v2) => {
                        return Err(VmFault::TypeMismatch {
                            expected: "numbers".to_string(),
                            actual: format!("{} and {}", v1.type_name(), v2.type_name()),
                        });
                    }
                },
                RegOpCode::Div => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        if *y == 0 {
                            return Err(VmFault::DivisionByZero);
                        }
                        #[allow(clippy::cast_precision_loss)]
                        let res = (*x as f64) / (*y as f64);
                        self.registers[a] = Value::Float(res);
                    }
                    (Value::Float(x), Value::Float(y)) => {
                        if *y == 0.0 {
                            return Err(VmFault::DivisionByZero);
                        }
                        self.registers[a] = Value::Float(x / y);
                    }
                    _ => return Err(VmFault::DivisionByZero),
                },
                RegOpCode::IntDiv => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        if *y == 0 {
                            return Err(VmFault::DivisionByZero);
                        }
                        self.registers[a] = Value::Int(x / y);
                    }
                    _ => return Err(VmFault::DivisionByZero),
                },
                RegOpCode::Mod => match (&self.registers[b], &self.registers[c]) {
                    (Value::Int(x), Value::Int(y)) => {
                        if *y == 0 {
                            return Err(VmFault::DivisionByZero);
                        }
                        self.registers[a] = Value::Int(x % y);
                    }
                    _ => return Err(VmFault::DivisionByZero),
                },
                RegOpCode::Neg => match &self.registers[b] {
                    Value::Int(x) => self.registers[a] = Value::Int(-x),
                    Value::Float(x) => self.registers[a] = Value::Float(-x),
                    other => {
                        return Err(VmFault::TypeMismatch {
                            expected: "number".to_string(),
                            actual: other.type_name().to_string(),
                        });
                    }
                },
                RegOpCode::Not => {
                    let b_val = self.registers[b].as_bool()?;
                    self.registers[a] = Value::Bool(!b_val);
                }
                RegOpCode::Equal => {
                    self.registers[a] = Value::Bool(self.registers[b] == self.registers[c]);
                }
                RegOpCode::NotEqual => {
                    self.registers[a] = Value::Bool(self.registers[b] != self.registers[c]);
                }
                RegOpCode::Less => {
                    self.registers[a] = self.registers[b].less(&self.registers[c])?;
                }
                RegOpCode::LessEqual => {
                    self.registers[a] = self.registers[b].less_equal(&self.registers[c])?;
                }
                RegOpCode::Greater => {
                    self.registers[a] = self.registers[b].greater(&self.registers[c])?;
                }
                RegOpCode::GreaterEqual => {
                    self.registers[a] = self.registers[b].greater_equal(&self.registers[c])?;
                }
                RegOpCode::Jump => {
                    let offset = inst.sbx();
                    let target = (self.pc as i32) + offset;
                    if target < 0 {
                        return Err(VmFault::CorruptedBytecode {
                            offset: self.pc - 1,
                            reason: format!("jump target {target} out of range"),
                        });
                    }
                    self.pc = target as usize;
                }
                RegOpCode::JumpIfTrue => {
                    if self.registers[a].as_bool()? {
                        let offset = inst.sbx();
                        let target = (self.pc as i32) + offset;
                        if target < 0 {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: format!("jump target {target} out of range"),
                            });
                        }
                        self.pc = target as usize;
                    }
                }
                RegOpCode::JumpIfFalse => {
                    if !self.registers[a].as_bool()? {
                        let offset = inst.sbx();
                        let target = (self.pc as i32) + offset;
                        if target < 0 {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: format!("jump target {target} out of range"),
                            });
                        }
                        self.pc = target as usize;
                    }
                }
                RegOpCode::JumpIfSetLocal => {
                    if !matches!(self.registers[a], Value::Unset) {
                        let offset = inst.sbx();
                        let target = (self.pc as i32) + offset;
                        if target < 0 {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: format!("jump target {target} out of range"),
                            });
                        }
                        self.pc = target as usize;
                    }
                }
                RegOpCode::GetGlobal => {
                    let bx = inst.bx() as usize;
                    if let Some(Value::String(name)) = self.constants.get(bx) {
                        let val = self
                            .globals
                            .get(name.as_str())
                            .cloned()
                            .unwrap_or(Value::None);
                        self.registers[a] = val;
                    }
                }
                RegOpCode::SetGlobal => {
                    let bx = inst.bx() as usize;
                    if let Some(Value::String(name)) = self.constants.get(bx) {
                        self.globals
                            .insert(name.to_string(), self.registers[a].clone());
                    }
                }
                RegOpCode::GetField => {
                    let bx = inst.bx() as usize;
                    let field_name = match self.constants.get(bx) {
                        Some(Value::String(s)) => s.as_str(),
                        _ => {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: "invalid field name constant".to_string(),
                            });
                        }
                    };
                    let target = self.registers[a].clone();
                    if let Value::Failure(err) = &target {
                        if field_name == "message" {
                            self.registers[a] = Value::String(Rc::new(err.message.clone()));
                            continue;
                        }
                        if field_name == "payload" {
                            self.registers[a] = err.payload.clone();
                            continue;
                        }
                    }
                    match target {
                        Value::Struct(inst) => {
                            let val =
                                inst.borrow()
                                    .get_field(field_name)
                                    .cloned()
                                    .ok_or_else(|| VmFault::NoSuchField {
                                        type_name: inst.borrow().type_name.clone(),
                                        field: field_name.to_string(),
                                    })?;
                            self.registers[a] = val;
                        }
                        Value::Dict(d) => {
                            let key = Value::String(Rc::new(field_name.to_string()));
                            let val = d.borrow().get(&key).cloned().unwrap_or(Value::None);
                            self.registers[a] = val;
                        }
                        other => {
                            return Err(VmFault::NoSuchField {
                                type_name: other.type_name().to_string(),
                                field: field_name.to_string(),
                            });
                        }
                    }
                }
                RegOpCode::SetField => {
                    let name_idx = b;
                    let val_reg = c;
                    let field_name = match self.constants.get(name_idx) {
                        Some(Value::String(s)) => s.as_str(),
                        _ => {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: "invalid field name constant".to_string(),
                            });
                        }
                    };
                    let new_val = self.registers[val_reg].clone();
                    match &self.registers[a] {
                        Value::Struct(inst) => {
                            inst.borrow_mut().set_field(field_name, new_val)?;
                        }
                        Value::Dict(d) => {
                            let key = Value::String(Rc::new(field_name.to_string()));
                            d.borrow_mut().upsert(key, new_val);
                        }
                        other => {
                            return Err(VmFault::NoSuchField {
                                type_name: other.type_name().to_string(),
                                field: field_name.to_string(),
                            });
                        }
                    }
                }
                RegOpCode::GetIndex => {
                    let target = self.registers[b].clone();
                    let key = &self.registers[c];
                    match (&target, key) {
                        (Value::List(l), Value::Int(i)) => {
                            let list = l.borrow();
                            let len = list.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if *i < 0 { (len as i64) + *i } else { *i };
                            let idx = usize::try_from(actual)
                                .ok()
                                .filter(|&idx| idx < len)
                                .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                            self.registers[a] = list[idx].clone();
                        }
                        (Value::Dict(d), k) => {
                            self.registers[a] = d.borrow().get(k).cloned().unwrap_or(Value::None);
                        }
                        (Value::Bytes(b), Value::Int(i)) => {
                            let bytes = b.borrow();
                            let len = bytes.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if *i < 0 { (len as i64) + *i } else { *i };
                            let idx = usize::try_from(actual)
                                .ok()
                                .filter(|&idx| idx < len)
                                .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                            self.registers[a] = Value::Byte(bytes[idx]);
                        }
                        (Value::String(s), Value::Int(i)) => {
                            let chars: Vec<char> = s.chars().collect();
                            let len = chars.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if *i < 0 { (len as i64) + *i } else { *i };
                            let idx = usize::try_from(actual)
                                .ok()
                                .filter(|&idx| idx < len)
                                .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                            self.registers[a] = Value::String(Rc::new(chars[idx].to_string()));
                        }
                        (other, _) => {
                            return Err(VmFault::TypeMismatch {
                                expected: "indexable collection".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    }
                }
                RegOpCode::SetIndex => {
                    let key = &self.registers[b];
                    let val = self.registers[c].clone();
                    match &self.registers[a] {
                        Value::List(l) => {
                            if let Value::Int(i) = key {
                                let mut list = l.borrow_mut();
                                let len = list.len();
                                #[allow(clippy::cast_possible_wrap)]
                                let actual = if *i < 0 { (len as i64) + *i } else { *i };
                                let idx = usize::try_from(actual)
                                    .ok()
                                    .filter(|&idx| idx < len)
                                    .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                                list[idx] = val;
                            } else {
                                return Err(VmFault::TypeMismatch {
                                    expected: "Int index".to_string(),
                                    actual: key.type_name().to_string(),
                                });
                            }
                        }
                        Value::Dict(d) => {
                            d.borrow_mut().upsert(key.clone(), val);
                        }
                        Value::Bytes(bytes) => {
                            if let (Value::Int(i), Value::Byte(b)) = (key, &val) {
                                let mut b_vec = bytes.borrow_mut();
                                let len = b_vec.len();
                                #[allow(clippy::cast_possible_wrap)]
                                let actual = if *i < 0 { (len as i64) + *i } else { *i };
                                let idx = usize::try_from(actual)
                                    .ok()
                                    .filter(|&idx| idx < len)
                                    .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                                b_vec[idx] = *b;
                            }
                        }
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "mutable collection".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    }
                }
                RegOpCode::Call => {
                    let callee = self.registers[a].clone();
                    let arg_count = b;
                    let args = self.registers[a + 1..a + 1 + arg_count].to_vec();
                    match callee {
                        Value::Native(native) => {
                            let result = (native.func)(&args)?;
                            self.registers[a] = result;
                        }
                        Value::BoundMethod(bm) => match bm.kind {
                            crate::value::MethodKind::Native(func) => {
                                let result = func(&bm.receiver, &args)?;
                                self.registers[a] = result;
                            }
                            _ => {
                                return Err(VmFault::NotCallable {
                                    type_name: format!("<method {}>", bm.name),
                                });
                            }
                        },
                        Value::Function {
                            entry_ip, arity, ..
                        } => {
                            if (arity as usize) != arg_count {
                                return Err(VmFault::TypeMismatch {
                                    expected: format!("{arity} arguments"),
                                    actual: format!("{arg_count} arguments"),
                                });
                            }
                            let func_idx = entry_ip as usize;
                            let func = self.functions.get(func_idx).cloned().ok_or_else(|| {
                                VmFault::CorruptedBytecode {
                                    offset: self.pc - 1,
                                    reason: format!("unknown function index {func_idx}"),
                                }
                            })?;
                            let result = self.invoke_function(&func, &args)?;
                            self.registers[a] = result;
                        }
                        other => {
                            return Err(VmFault::NotCallable {
                                type_name: other.type_name().to_string(),
                            });
                        }
                    }
                }
                RegOpCode::CallPipe => {
                    let callee = self.registers[c].clone();
                    let stream_arg = self.registers[b].clone();
                    match callee {
                        Value::Native(native) => {
                            let result = (native.func)(&[stream_arg])?;
                            self.registers[a] = result;
                        }
                        Value::BoundMethod(bm) => match bm.kind {
                            crate::value::MethodKind::Native(func) => {
                                let result = func(&bm.receiver, &[stream_arg])?;
                                self.registers[a] = result;
                            }
                            _ => {
                                return Err(VmFault::NotCallable {
                                    type_name: format!("<method {}>", bm.name),
                                });
                            }
                        },
                        Value::Function {
                            entry_ip, arity, ..
                        } => {
                            if arity as usize != 1 {
                                return Err(VmFault::TypeMismatch {
                                    expected: "1 argument for piped call".to_string(),
                                    actual: format!("function arity {arity}"),
                                });
                            }
                            let func_idx = entry_ip as usize;
                            let func = self.functions.get(func_idx).cloned().ok_or_else(|| {
                                VmFault::CorruptedBytecode {
                                    offset: self.pc - 1,
                                    reason: format!("unknown function index {func_idx}"),
                                }
                            })?;
                            let result = self.invoke_function(&func, &[stream_arg])?;
                            self.registers[a] = result;
                        }
                        other => {
                            return Err(VmFault::NotCallable {
                                type_name: other.type_name().to_string(),
                            });
                        }
                    }
                }
                RegOpCode::Return => {
                    return Ok(self.registers[a].clone());
                }
                RegOpCode::MakeFunction => {
                    let bx = inst.bx() as usize;
                    if let Some(func) = self.functions.get(bx) {
                        let arity = u16::try_from(func.arity).unwrap_or(u16::MAX);
                        self.registers[a] = Value::Function {
                            entry_ip: bx as u32,
                            arity,
                            is_async: func.is_async,
                        };
                    } else if let Some(Value::Function {
                        entry_ip,
                        arity,
                        is_async,
                    }) = self.constants.get(bx).cloned()
                    {
                        self.registers[a] = Value::Function {
                            entry_ip,
                            arity,
                            is_async,
                        };
                    } else {
                        return Err(VmFault::CorruptedBytecode {
                            offset: self.pc - 1,
                            reason: format!("unknown function index {bx}"),
                        });
                    }
                }
                RegOpCode::NewList => {
                    let start = b;
                    let count = c;
                    if start + count > 256 {
                        return Err(VmFault::StackUnderflow);
                    }
                    let items = self.registers[start..start + count].to_vec();
                    self.registers[a] = Value::List(Rc::new(RefCell::new(items)));
                }
                RegOpCode::NewDict => {
                    let start = b;
                    let count = c;
                    if start + count * 2 > 256 {
                        return Err(VmFault::StackUnderflow);
                    }
                    let mut entries = Vec::with_capacity(count);
                    for i in 0..count {
                        let k = self.registers[start + i * 2].clone();
                        let v = self.registers[start + i * 2 + 1].clone();
                        entries.push((k, v));
                    }
                    self.registers[a] =
                        Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))));
                }
                RegOpCode::NewStruct => {
                    let type_c_idx = b;
                    let field_count = c;
                    let type_name = match self.constants.get(type_c_idx) {
                        Some(Value::String(s)) => s.clone(),
                        _ => {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: "invalid struct type name constant".to_string(),
                            });
                        }
                    };
                    if a + field_count > 256 {
                        return Err(VmFault::StackUnderflow);
                    }
                    let values = self.registers[a..a + field_count].to_vec();
                    let instance = if let Some(defs) = self.struct_defs.get(type_name.as_str()) {
                        let mut fields = Vec::with_capacity(defs.len());
                        let mut fixed_fields = HashSet::new();
                        for ((f_name, is_fixed), val) in defs.iter().cloned().zip(values) {
                            if is_fixed {
                                fixed_fields.insert(f_name.clone());
                            }
                            fields.push((f_name, val));
                        }
                        StructInstance {
                            type_name: (*type_name).clone(),
                            fields,
                            fixed_fields,
                            under_construction: false,
                        }
                    } else {
                        let fields = values
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (format!("field_{i}"), v))
                            .collect();
                        StructInstance {
                            type_name: (*type_name).clone(),
                            fields,
                            fixed_fields: HashSet::new(),
                            under_construction: false,
                        }
                    };
                    self.registers[a] = Value::Struct(Rc::new(RefCell::new(instance)));
                }
                RegOpCode::Range => {
                    let start = match self.registers[b] {
                        Value::Int(n) => n,
                        ref other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "Int".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    let end = match self.registers[c] {
                        Value::Int(n) => n,
                        ref other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "Int".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    self.registers[a] = Value::range(start, end);
                }
                RegOpCode::CloneStruct => {
                    let val = self.registers[b].clone();
                    match val {
                        Value::Struct(inst) => {
                            let cloned = inst.borrow().clone();
                            self.registers[a] = Value::Struct(Rc::new(RefCell::new(cloned)));
                        }
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "struct".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    }
                }
                RegOpCode::IsVariant => {
                    let bx = inst.bx() as usize;
                    let expected = match self.constants.get(bx) {
                        Some(Value::String(s)) => s.as_str(),
                        _ => {
                            return Err(VmFault::CorruptedBytecode {
                                offset: self.pc - 1,
                                reason: "invalid variant name constant".to_string(),
                            });
                        }
                    };
                    let val = &self.registers[a];
                    let matches = match val {
                        Value::Struct(inst) => {
                            let t = &inst.borrow().type_name;
                            t == expected || t.ends_with(&format!(".{expected}"))
                        }
                        _ => false,
                    };
                    self.registers[a] = Value::Bool(matches);
                }
                RegOpCode::Len => {
                    let val = &self.registers[b];
                    let len = match val {
                        Value::String(s) => s.chars().count() as i64,
                        Value::List(l) => l.borrow().len() as i64,
                        Value::Dict(d) => d.borrow().len() as i64,
                        Value::Set(s) => s.borrow().len() as i64,
                        Value::Bytes(b) => b.borrow().len() as i64,
                        Value::Range(r) => (r.end - r.start).max(0),
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "collection, string or range".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    self.registers[a] = Value::Int(len);
                }
                RegOpCode::IterAt => {
                    let coll = &self.registers[b];
                    let idx_reg = c >> 2;
                    let mode = (c & 3) as u8;
                    let idx_val = &self.registers[idx_reg];
                    let ordinal = match idx_val {
                        Value::Int(n) => *n,
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "Int iteration index".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    let projected = match coll {
                        Value::List(l) => {
                            let list = l.borrow();
                            let len = list.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if ordinal < 0 {
                                (len as i64) + ordinal
                            } else {
                                ordinal
                            };
                            let pos = usize::try_from(actual).ok().filter(|&p| p < len).ok_or(
                                VmFault::IndexOutOfRange {
                                    index: ordinal,
                                    len,
                                },
                            )?;
                            if mode == 1 {
                                Value::Int(actual)
                            } else {
                                list[pos].clone()
                            }
                        }
                        Value::Dict(d) => {
                            let dict = d.borrow();
                            let len = dict.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if ordinal < 0 {
                                (len as i64) + ordinal
                            } else {
                                ordinal
                            };
                            let pos = usize::try_from(actual).ok().filter(|&p| p < len).ok_or(
                                VmFault::IndexOutOfRange {
                                    index: ordinal,
                                    len,
                                },
                            )?;
                            let (k, v) = &dict.entries()[pos];
                            if mode == 1 { k.clone() } else { v.clone() }
                        }
                        Value::Range(r) => {
                            let len = (r.end - r.start).max(0) as usize;
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if ordinal < 0 {
                                (len as i64) + ordinal
                            } else {
                                ordinal
                            };
                            let pos = usize::try_from(actual).ok().filter(|&p| p < len).ok_or(
                                VmFault::IndexOutOfRange {
                                    index: ordinal,
                                    len,
                                },
                            )?;
                            Value::Int(r.start + (pos as i64))
                        }
                        Value::String(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            let len = chars.len();
                            #[allow(clippy::cast_possible_wrap)]
                            let actual = if ordinal < 0 {
                                (len as i64) + ordinal
                            } else {
                                ordinal
                            };
                            let pos = usize::try_from(actual).ok().filter(|&p| p < len).ok_or(
                                VmFault::IndexOutOfRange {
                                    index: ordinal,
                                    len,
                                },
                            )?;
                            if mode == 1 {
                                Value::Int(actual)
                            } else {
                                Value::String(Rc::new(chars[pos].to_string()))
                            }
                        }
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "iterable collection".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    self.registers[a] = projected;
                }
                RegOpCode::TypeIs => {
                    let val = &self.registers[b];
                    let tag = &self.registers[c];
                    let matches = match tag {
                        Value::Type(tag) => tag.matches(val),
                        Value::UserType(target_name) => match val {
                            Value::Struct(inst) => {
                                let name = &inst.borrow().type_name;
                                name == target_name.as_str()
                                    || name
                                        .split_once('.')
                                        .is_some_and(|(parent, _)| parent == target_name.as_str())
                            }
                            _ => false,
                        },
                        Value::Struct(target_inst) => match val {
                            Value::Struct(inst) => {
                                inst.borrow().type_name == target_inst.borrow().type_name
                            }
                            _ => false,
                        },
                        Value::String(s) => match val {
                            Value::Struct(inst) => inst.borrow().type_name == **s,
                            _ => false,
                        },
                        _ => false,
                    };
                    self.registers[a] = Value::Bool(matches);
                }
                RegOpCode::TypeIsNullable => {
                    let val = &self.registers[b];
                    let tag = &self.registers[c];
                    let matches = if matches!(val, Value::None) {
                        true
                    } else {
                        match tag {
                            Value::Type(tag) => tag.matches(val),
                            Value::UserType(target_name) => match val {
                                Value::Struct(inst) => {
                                    let name = &inst.borrow().type_name;
                                    name == target_name.as_str()
                                        || name.split_once('.').is_some_and(|(parent, _)| {
                                            parent == target_name.as_str()
                                        })
                                }
                                _ => false,
                            },
                            Value::Struct(target_inst) => match val {
                                Value::Struct(inst) => {
                                    inst.borrow().type_name == target_inst.borrow().type_name
                                }
                                _ => false,
                            },
                            Value::String(s) => match val {
                                Value::Struct(inst) => inst.borrow().type_name == **s,
                                _ => false,
                            },
                            _ => false,
                        }
                    };
                    self.registers[a] = Value::Bool(matches);
                }
                RegOpCode::PushHandler => {
                    let offset = inst.sbx();
                    let target = (self.pc as i32) + offset;
                    if target < 0 {
                        return Err(VmFault::CorruptedBytecode {
                            offset: self.pc - 1,
                            reason: format!("handler target {target} out of range"),
                        });
                    }
                    self.handler_stack.push((target as usize, a as u8));
                }
                RegOpCode::PopHandler => {
                    self.handler_stack.pop();
                }
                RegOpCode::Fail => {
                    let val = &self.registers[a];
                    let (msg, payload) = match val {
                        Value::String(s) => ((**s).clone(), Value::None),
                        Value::Failure(f) => (f.message.clone(), f.payload.clone()),
                        other => (other.to_string(), other.clone()),
                    };
                    let failure = Value::failure_with_payload(msg, payload);
                    if let Some((handler_pc, err_reg)) = self.handler_stack.pop() {
                        self.registers[err_reg as usize] = failure;
                        self.pc = handler_pc;
                    } else {
                        self.registers[a] = failure;
                    }
                }
                RegOpCode::PropagateFailure => {
                    if self.registers[a].is_failure() {
                        let failure = self.registers[a].clone();
                        if let Some((handler_pc, err_reg)) = self.handler_stack.pop() {
                            self.registers[err_reg as usize] = failure;
                            self.pc = handler_pc;
                        } else {
                            return Ok(failure);
                        }
                    }
                }
            }
        }
        Ok(Value::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_vm_arithmetic_pipeline() {
        let mut vm = RegVm::new();
        let code = vec![
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 10), // R[1] = 10
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 2, 25), // R[2] = 25
            RegInstruction::encode_abc(RegOpCode::Add, 3, 1, 2),    // R[3] = R[1] + R[2] (35)
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 4, 5),  // R[4] = 5
            RegInstruction::encode_abc(RegOpCode::Mul, 0, 3, 4),    // R[0] = R[3] * R[4] (175)
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0), // return R[0]
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(175));
    }

    #[test]
    fn test_register_vm_branching() {
        let mut vm = RegVm::new();
        let code = vec![
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 0, 10),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 20),
            RegInstruction::encode_abc(RegOpCode::Less, 2, 0, 1), // R[2] = 10 < 20 (true)
            RegInstruction::encode_asbx(RegOpCode::JumpIfFalse, 2, 2), // skip if false
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 0, 999), // R[0] = 999
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0), // return R[0]
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 0, -1), // R[0] = -1
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0), // return R[0]
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(999));
    }

    #[test]
    fn test_register_vm_size_in_memory() {
        // 256 registers of 16 bytes = 4096 bytes (exactly 4 KiB!).
        assert_eq!(std::mem::size_of::<[Value; 256]>(), 4096);
    }

    #[test]
    fn test_reg_emitter_to_reg_vm_end_to_end() {
        use aipo_bytecode::RegEmitter;
        use aipo_ir::{BinaryOp, CoreFunction, CoreInst};
        use aipo_source::SourceSpan;

        let span = SourceSpan::default();
        let func = CoreFunction {
            name: "calculate".to_string(),
            is_async: false,
            params: vec!["a".to_string(), "b".to_string()],
            locals: vec!["res".to_string()],
            upvalues: Vec::new(),
            instructions: vec![
                CoreInst::Load("a".to_string(), span),
                CoreInst::Load("b".to_string(), span),
                CoreInst::Binary(BinaryOp::Mul, span),
                CoreInst::Store("res".to_string(), span),
                CoreInst::Load("res".to_string(), span),
                CoreInst::Return {
                    has_value: true,
                    span,
                },
            ],
            span,
        };

        let compiled = RegEmitter::new().compile_function(&func);
        let mut vm = RegVm::new();
        // Set parameters R[0] and R[1]
        vm.registers[0] = Value::Int(6);
        vm.registers[1] = Value::Int(7);

        let outcome = vm.run_function(&compiled).unwrap();
        assert_eq!(outcome, Value::Int(42));
    }

    #[test]
    fn test_register_vm_call_native_and_indexing() {
        let mut vm = RegVm::new();
        let double_native = Value::native("double", 1, |args| match args.first() {
            Some(Value::Int(n)) => Ok(Value::Int(n * 2)),
            _ => Ok(Value::None),
        });
        vm.registers[10] = double_native;
        vm.registers[11] = Value::Int(21);

        let code = vec![
            // R[10] = call R[10] with 1 argument (R[11])
            RegInstruction::encode_abc(RegOpCode::Call, 10, 1, 1),
            // return R[10]
            RegInstruction::encode_abc(RegOpCode::Return, 10, 0, 0),
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_register_vm_list_get_and_set_index() {
        use std::cell::RefCell;
        let mut vm = RegVm::new();
        let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(10), Value::Int(20)])));
        vm.registers[0] = list;
        vm.registers[1] = Value::Int(1); // index 1
        vm.registers[2] = Value::Int(99); // new value

        let code = vec![
            // SetIndex: R[0][R[1]] = R[2] (list[1] = 99)
            RegInstruction::encode_abc(RegOpCode::SetIndex, 0, 1, 2),
            // GetIndex: R[5] = R[0][R[1]]
            RegInstruction::encode_abc(RegOpCode::GetIndex, 5, 0, 1),
            // return R[5]
            RegInstruction::encode_abc(RegOpCode::Return, 5, 0, 0),
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_register_vm_user_function_call() {
        use aipo_bytecode::RegCompiledFunction;
        // adder: R[0] = R[0] + R[1]; return R[0]
        let adder = RegCompiledFunction {
            name: "adder".to_string(),
            arity: 2,
            is_async: false,
            instructions: vec![
                RegInstruction::encode_abc(RegOpCode::Add, 0, 0, 1),
                RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0),
            ],
            constants: vec![],
            num_registers: 2,
        };
        let mut vm = RegVm::new();
        vm.functions = vec![adder];
        vm.registers[0] = Value::Function {
            entry_ip: 0,
            arity: 2,
            is_async: false,
        };
        vm.registers[1] = Value::Int(6);
        vm.registers[2] = Value::Int(7);

        let code = vec![
            RegInstruction::encode_abc(RegOpCode::Call, 0, 2, 1),
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0),
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(13));
    }

    #[test]
    fn test_register_vm_makefunction_table_call() {
        use aipo_bytecode::RegCompiledFunction;
        // double: R[0] = R[0] + R[0]; return R[0]
        let double = RegCompiledFunction {
            name: "double".to_string(),
            arity: 1,
            is_async: false,
            instructions: vec![
                RegInstruction::encode_abc(RegOpCode::Add, 0, 0, 0),
                RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0),
            ],
            constants: vec![],
            num_registers: 1,
        };
        let mut vm = RegVm::new();
        vm.functions = vec![double];

        let code = vec![
            RegInstruction::encode_abx(RegOpCode::MakeFunction, 0, 0),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 21),
            RegInstruction::encode_abc(RegOpCode::Call, 0, 1, 1),
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0),
        ];

        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_register_vm_call_depth_limit() {
        use aipo_bytecode::RegCompiledFunction;
        // recurse: R[1] = R[0]; call R[1] with 1 arg; return R[1]
        let recurse = RegCompiledFunction {
            name: "recurse".to_string(),
            arity: 1,
            is_async: false,
            instructions: vec![
                RegInstruction::encode_abx(RegOpCode::MakeFunction, 1, 0),
                RegInstruction::encode_abc(RegOpCode::Move, 2, 0, 0),
                RegInstruction::encode_abc(RegOpCode::Call, 1, 1, 1),
                RegInstruction::encode_abc(RegOpCode::Return, 1, 0, 0),
            ],
            constants: vec![],
            num_registers: 3,
        };
        let mut vm = RegVm::new();
        vm.max_call_depth = 8;
        vm.functions = vec![recurse];
        vm.registers[0] = Value::Function {
            entry_ip: 0,
            arity: 1,
            is_async: false,
        };
        vm.registers[1] = Value::Int(0);

        let code = vec![
            RegInstruction::encode_abc(RegOpCode::Call, 0, 1, 1),
            RegInstruction::encode_abc(RegOpCode::Return, 0, 0, 0),
        ];

        let err = vm.run(&code).unwrap_err();
        assert!(matches!(err, VmFault::CorruptedBytecode { .. }));
    }

    #[test]
    fn test_register_vm_new_list_and_len() {
        let mut vm = RegVm::new();
        let code = vec![
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 10),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 2, 20),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 3, 30),
            // R[0] = [R[1], R[2], R[3]]
            RegInstruction::encode_abc(RegOpCode::NewList, 0, 1, 3),
            // R[4] = len(R[0])
            RegInstruction::encode_abc(RegOpCode::Len, 4, 0, 0),
            RegInstruction::encode_abc(RegOpCode::Return, 4, 0, 0),
        ];
        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_register_vm_new_dict() {
        let mut vm = RegVm::new();
        vm.constants = vec![
            Value::String(Rc::new("alpha".to_string())),
            Value::String(Rc::new("beta".to_string())),
        ];
        let code = vec![
            RegInstruction::encode_abx(RegOpCode::LoadConst, 1, 0),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 2, 100),
            RegInstruction::encode_abx(RegOpCode::LoadConst, 3, 1),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 4, 200),
            // R[0] = { R[1]: R[2], R[3]: R[4] }
            RegInstruction::encode_abc(RegOpCode::NewDict, 0, 1, 2),
            // GetIndex: R[5] = R[0][R[1]]
            RegInstruction::encode_abc(RegOpCode::GetIndex, 5, 0, 1),
            RegInstruction::encode_abc(RegOpCode::Return, 5, 0, 0),
        ];
        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_register_vm_new_struct_and_clone() {
        let mut vm = RegVm::new();
        vm.constants = vec![
            Value::String(Rc::new("Point".to_string())),
            Value::String(Rc::new("x".to_string())),
            Value::String(Rc::new("y".to_string())),
        ];
        vm.struct_defs.insert(
            "Point".to_string(),
            vec![("x".to_string(), true), ("y".to_string(), false)],
        );

        let code = vec![
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 0, 10),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 20),
            // R[0] = Point{ x: R[0], y: R[1] }
            RegInstruction::encode_abc(RegOpCode::NewStruct, 0, 0, 2),
            // Clone R[0] into R[1]
            RegInstruction::encode_abc(RegOpCode::CloneStruct, 1, 0, 0),
            // GetField 'x' from R[1] into R[2]
            RegInstruction::encode_abx(RegOpCode::GetField, 1, 1),
            RegInstruction::encode_abc(RegOpCode::Return, 1, 0, 0),
        ];
        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_register_vm_range_and_variant() {
        let mut vm = RegVm::new();
        vm.constants = vec![
            Value::String(Rc::new("Option.Some".to_string())),
            Value::String(Rc::new("val".to_string())),
        ];
        vm.struct_defs
            .insert("Option.Some".to_string(), vec![("val".to_string(), true)]);

        let code = vec![
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 1, 5),
            RegInstruction::encode_asbx(RegOpCode::LoadInt, 2, 10),
            // R[0] = 5..10
            RegInstruction::encode_abc(RegOpCode::Range, 0, 1, 2),
            // R[3] = len(R[0])
            RegInstruction::encode_abc(RegOpCode::Len, 3, 0, 0),
            // Construct Option.Some
            RegInstruction::encode_abc(RegOpCode::NewStruct, 1, 0, 1),
            // IsVariant check
            RegInstruction::encode_abx(RegOpCode::IsVariant, 1, 0),
            RegInstruction::encode_abc(RegOpCode::Return, 1, 0, 0),
        ];
        let result = vm.run(&code).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}
