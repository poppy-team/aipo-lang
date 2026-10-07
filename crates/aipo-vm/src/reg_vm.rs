//! Virtual register execution engine for 32-bit instructions (Marco 3 / ADP-014).

use crate::arena::ArenaAllocator;
use crate::fault::VmFault;
use crate::value::{Value, check_safe_int};
use aipo_bytecode::{Constant, RegCompiledFunction, RegCompiledModule, RegInstruction, RegOpCode};
use std::collections::HashMap;
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
    /// Active nested-call frames (depth guard).
    pub frames: Vec<RegCallFrame>,
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
            frames: Vec::new(),
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
    }

    /// Executes a compiled register module's top-level script.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error occurs.
    pub fn run_module(&mut self, module: &RegCompiledModule) -> Result<Value, VmFault> {
        self.functions = module.functions.clone();
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
}
