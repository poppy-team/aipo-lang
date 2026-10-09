//! Virtual register execution engine for 32-bit instructions (Marco 3 / ADP-014).

use crate::arena::ArenaAllocator;
use crate::convert::{TypeTag, convert_via_type};
use crate::fault::VmFault;
use crate::value::{DictMap, StructInstance, Value, check_safe_int, string_char_at};
use aipo_bytecode::{Constant, RegCompiledFunction, RegCompiledModule, RegInstruction, RegOpCode};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Maximum nested user-function call depth (bounds Rust stack + RAM).
pub const REG_MAX_CALL_DEPTH: usize = 64;

struct VerificationScope {
    flag: Rc<Cell<bool>>,
    previous: bool,
}
impl Drop for VerificationScope {
    fn drop(&mut self) {
        self.flag.set(self.previous);
    }
}

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
    /// 256 virtual registers; measure target layout before assuming a byte size.
    pub registers: [Value; 256],
    /// Constant pool.
    pub constants: Vec<Value>,
    /// Global variables environment.
    pub globals: HashMap<String, Value>,
    /// Compiled user-function table for `MakeFunction`/`Call`.
    pub functions: Vec<Rc<RegCompiledFunction>>,
    /// Declared struct definitions: name -> fields.
    pub struct_defs: HashMap<String, Vec<(String, bool)>>,
    /// Active nested-call frames (depth guard).
    pub frames: Vec<RegCallFrame>,
    /// Active exception/failure handler stack: `(handler_pc, err_reg)`.
    pub handler_stack: Vec<(usize, u8, usize)>,
    /// Maximum nested-call depth.
    pub max_call_depth: usize,
    /// Reserved bump arena; current managed values still use Rc allocations.
    pub arena: ArenaAllocator,
    /// Program counter.
    pub pc: usize,
    active_registers: usize,
    verified_module: Rc<Cell<bool>>,
    function_constants: Vec<Vec<Value>>,
    max_instructions: Option<u64>,
    instruction_count: u64,
    iteration_guards: Vec<(Value, u64)>,
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
            active_registers: 256,
            verified_module: Rc::new(Cell::new(false)),
            function_constants: Vec::new(),
            max_instructions: None,
            instruction_count: 0,
            iteration_guards: Vec::new(),
        }
    }

    /// Sets a cumulative bytecode budget, resetting its consumed instruction count.
    /// `None` disables accounting; native callbacks are outside this budget.
    pub fn set_instruction_budget(&mut self, limit: Option<u64>) {
        self.max_instructions = limit;
        self.instruction_count = 0;
    }

    /// Number of bytecode instructions consumed while budgeting is enabled.
    #[must_use]
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Resets consumption without changing the configured limit.
    pub fn reset_instruction_count(&mut self) {
        self.instruction_count = 0;
    }

    fn corrupted(&self, reason: impl Into<String>) -> VmFault {
        VmFault::CorruptedBytecode {
            offset: self.pc.saturating_sub(1),
            reason: reason.into(),
        }
    }

    fn checked_jump(&self, offset: i32, length: usize) -> Result<usize, VmFault> {
        self.pc
            .checked_add_signed(offset as isize)
            .filter(|target| *target < length)
            .ok_or_else(|| self.corrupted("jump target outside instruction stream"))
    }

    fn constant_name(&self, index: usize) -> Result<String, VmFault> {
        match self.constants.get(index) {
            Some(Value::String(name)) => Ok(name.to_string()),
            _ => Err(self.corrupted(format!("invalid string constant at {index}"))),
        }
    }

    fn check_arity(expected: usize, actual: usize) -> Result<(), VmFault> {
        if expected == usize::MAX || expected == actual {
            Ok(())
        } else {
            Err(VmFault::TypeMismatch {
                expected: format!("{expected} arguments"),
                actual: format!("{actual} arguments"),
            })
        }
    }

    fn check_function(&self, func: &RegCompiledFunction) -> Result<(), VmFault> {
        if func.is_async
            || func.num_registers == 0
            || func.num_registers > 256
            || func.arity > func.num_registers
        {
            return Err(self.corrupted("unsupported async function or invalid register metadata"));
        }
        Ok(())
    }

    fn guarded_revision(value: &Value) -> u64 {
        match value {
            Value::List(items) => items.borrow().revision(),
            Value::Dict(items) => items.borrow().revision(),
            Value::Bytes(items) => items.borrow().revision(),
            Value::Set(items) => items.borrow().revision(),
            _ => 0,
        }
    }

    fn check_iteration_guards(&self) -> Result<(), VmFault> {
        if self
            .iteration_guards
            .iter()
            .any(|(value, len)| Self::guarded_revision(value) != *len)
        {
            Err(VmFault::MutationDuringIteration)
        } else {
            Ok(())
        }
    }

    fn check_dict_insert(&self, dict: &Rc<RefCell<DictMap>>, key: &Value) -> Result<(), VmFault> {
        let guarded = self
            .iteration_guards
            .iter()
            .any(|(value, _)| matches!(value, Value::Dict(other) if Rc::ptr_eq(dict, other)));
        if guarded && dict.borrow().get(key).is_none() {
            Err(VmFault::MutationDuringIteration)
        } else {
            Ok(())
        }
    }

    fn position(len: usize, index: i64) -> Result<usize, VmFault> {
        let position = if index < 0 {
            usize::try_from(index.unsigned_abs())
                .ok()
                .and_then(|distance| len.checked_sub(distance))
        } else {
            usize::try_from(index).ok()
        };
        position
            .filter(|position| *position < len)
            .ok_or(VmFault::IndexOutOfRange { index, len })
    }

    fn range_len(start: i64, end: i64) -> Result<i64, VmFault> {
        end.checked_sub(start)
            .ok_or_else(|| VmFault::Overflow {
                details: "range length overflow".into(),
            })
            .and_then(|length| check_safe_int(length.max(0)))
    }

    fn contract_holds(&self, type_name: &str, nullable: bool, value: &Value) -> bool {
        if matches!(value, Value::None) {
            return nullable;
        }
        if type_name == "Function" {
            return matches!(
                value,
                Value::Function { .. } | Value::Native(_) | Value::BoundMethod(_)
            );
        }
        if let Some(tag) = TypeTag::from_name(type_name) {
            return tag.matches(value);
        }
        if self.struct_defs.contains_key(type_name)
            || self.struct_defs.keys().any(|name| {
                name.split_once('.')
                    .is_some_and(|(parent, _)| parent == type_name)
            })
        {
            return matches!(value, Value::Struct(instance) if {
                let instance = instance.borrow();
                instance.type_name == type_name || instance.type_name.split_once('.').is_some_and(|(parent, _)| parent == type_name)
            });
        }
        // An empty unresolved interface imposes no operations. Nonempty interface
        // contracts are rejected by the register emitter until method parity exists.
        true
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
        self.functions = module.functions.iter().cloned().map(Rc::new).collect();
        self.function_constants = module
            .functions
            .iter()
            .map(|function| {
                function
                    .constants
                    .iter()
                    .map(Self::convert_constant)
                    .collect()
            })
            .collect();
        self.struct_defs = module.struct_defs.clone();
        for name in module.struct_defs.keys() {
            self.globals
                .entry(name.clone())
                .or_insert_with(|| Value::UserType(Rc::new(name.clone())));
            if let Some((parent, _)) = name.split_once('.') {
                self.globals
                    .entry(parent.to_string())
                    .or_insert_with(|| Value::UserType(Rc::new(parent.to_string())));
            }
        }
    }

    /// Executes a compiled register module's top-level script.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error occurs.
    pub fn run_module(&mut self, module: &RegCompiledModule) -> Result<Value, VmFault> {
        aipo_bytecode::RegVerifier::verify(module).map_err(|reason| self.corrupted(reason))?;
        self.load_module(module);
        self.frames.clear();
        let _scope = VerificationScope {
            flag: Rc::clone(&self.verified_module),
            previous: self.verified_module.replace(true),
        };
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
        self.check_function(func)?;
        aipo_bytecode::RegVerifier::verify_function(func, self.functions.len())
            .map_err(|reason| self.corrupted(reason))?;
        let constants = func.constants.iter().map(Self::convert_constant).collect();
        self.invoke_activation(func, args, constants)
    }

    fn invoke_index(&mut self, index: usize, args: &[Value]) -> Result<Value, VmFault> {
        let func = self
            .functions
            .get(index)
            .cloned()
            .ok_or_else(|| self.corrupted("invalid function index"))?;
        let constants = if self.verified_module.get() {
            self.function_constants
                .get(index)
                .cloned()
                .ok_or_else(|| self.corrupted("invalid function constant pool"))?
        } else {
            aipo_bytecode::RegVerifier::verify_function(&func, self.functions.len())
                .map_err(|reason| self.corrupted(reason))?;
            func.constants.iter().map(Self::convert_constant).collect()
        };
        self.invoke_activation(&func, args, constants)
    }

    fn invoke_activation(
        &mut self,
        func: &RegCompiledFunction,
        args: &[Value],
        constants: Vec<Value>,
    ) -> Result<Value, VmFault> {
        Self::check_arity(func.arity, args.len())?;
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
        let saved_active = self.active_registers;
        let window = saved_active.max(func.num_registers);
        let saved_registers: Vec<_> = self.registers[..window]
            .iter_mut()
            .map(|value| std::mem::replace(value, Value::None))
            .collect();
        self.active_registers = func.num_registers;
        let saved_constants = std::mem::take(&mut self.constants);
        let saved_pc = self.pc;
        for (i, arg) in args.iter().enumerate() {
            self.registers[i] = arg.clone();
        }
        self.constants = constants;
        let outcome = self.run_verified(&func.instructions);
        for (slot, value) in self.registers.iter_mut().zip(saved_registers) {
            *slot = value;
        }
        self.active_registers = saved_active;
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
        self.check_function(func)?;
        aipo_bytecode::RegVerifier::verify_function(func, self.functions.len())
            .map_err(|reason| self.corrupted(reason))?;
        self.constants = func.constants.iter().map(Self::convert_constant).collect();
        self.frames.clear();
        self.active_registers = func.num_registers;
        self.run_verified(&func.instructions)
    }

    /// Executes instructions sequentially until `Return` or error.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error (e.g. division by zero, overflow) occurs.
    pub fn run(&mut self, code: &[RegInstruction]) -> Result<Value, VmFault> {
        self.verified_module.set(false);
        self.active_registers = 256;
        let constants = self
            .constants
            .iter()
            .map(|value| match value {
                Value::String(text) => Constant::String((**text).clone()),
                _ => Constant::Nil,
            })
            .collect::<Vec<_>>();
        aipo_bytecode::RegVerifier::verify_raw(code, &constants, self.functions.len())
            .map_err(|reason| self.corrupted(reason))?;
        self.run_verified(code)
    }

    fn run_verified(&mut self, code: &[RegInstruction]) -> Result<Value, VmFault> {
        let saved_handlers = std::mem::take(&mut self.handler_stack);
        let guard_base = self.iteration_guards.len();
        let outcome = self.run_inner(code, guard_base).and_then(|value| {
            self.check_iteration_guards()?;
            Ok(value)
        });
        self.handler_stack = saved_handlers;
        self.iteration_guards.truncate(guard_base);
        outcome
    }

    fn run_inner(&mut self, code: &[RegInstruction], guard_base: usize) -> Result<Value, VmFault> {
        self.pc = 0;
        while self.pc < code.len() {
            self.check_iteration_guards()?;
            if let Some(limit) = self.max_instructions {
                if self.instruction_count >= limit {
                    return Err(VmFault::Overflow {
                        details: format!("register instruction budget {limit} exhausted"),
                    });
                }
                self.instruction_count += 1;
            }
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

            // B is 9 bits, but only 256 registers exist. Immediate/pool/count B
            // operands deliberately bypass this register-role check.
            if b >= self.registers.len()
                && matches!(
                    op,
                    RegOpCode::Move
                        | RegOpCode::Add
                        | RegOpCode::Sub
                        | RegOpCode::Mul
                        | RegOpCode::Div
                        | RegOpCode::IntDiv
                        | RegOpCode::Mod
                        | RegOpCode::Neg
                        | RegOpCode::Not
                        | RegOpCode::Equal
                        | RegOpCode::NotEqual
                        | RegOpCode::Less
                        | RegOpCode::LessEqual
                        | RegOpCode::Greater
                        | RegOpCode::GreaterEqual
                        | RegOpCode::CallPipe
                        | RegOpCode::GetIndex
                        | RegOpCode::SetIndex
                        | RegOpCode::IterAt
                        | RegOpCode::IterPrimary
                        | RegOpCode::IterKey
                        | RegOpCode::IterValue
                        | RegOpCode::NewList
                        | RegOpCode::NewDict
                        | RegOpCode::Range
                        | RegOpCode::Len
                        | RegOpCode::CloneStruct
                        | RegOpCode::TypeIs
                        | RegOpCode::TypeIsNullable
                )
            {
                return Err(self.corrupted(format!("register B={b} exceeds register file")));
            }

            match op {
                RegOpCode::Nop => {}
                RegOpCode::Move => {
                    self.registers[a] = self.registers[b].clone();
                }
                RegOpCode::LoadConst => {
                    let bx = inst.bx() as usize;
                    let val =
                        self.constants.get(bx).cloned().ok_or_else(|| {
                            self.corrupted(format!("invalid constant index {bx}"))
                        })?;
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
                RegOpCode::Add => {
                    self.registers[a] = self.registers[b].add(&self.registers[c])?;
                }
                RegOpCode::Sub => {
                    self.registers[a] = self.registers[b].sub(&self.registers[c])?;
                }
                RegOpCode::Mul => {
                    self.registers[a] = self.registers[b].mul(&self.registers[c])?;
                }
                RegOpCode::Div => {
                    self.registers[a] = self.registers[b].div(&self.registers[c])?;
                }
                RegOpCode::IntDiv => {
                    self.registers[a] = self.registers[b].int_div(&self.registers[c])?;
                }
                RegOpCode::Mod => {
                    self.registers[a] = self.registers[b].modulo(&self.registers[c])?;
                }
                RegOpCode::Neg => {
                    self.registers[a] = self.registers[b].negate()?;
                }
                RegOpCode::Not => {
                    self.registers[a] = self.registers[b].not()?;
                }
                RegOpCode::Equal => {
                    self.registers[a] = self.registers[b].equal(&self.registers[c])?;
                }
                RegOpCode::NotEqual => {
                    self.registers[a] = self.registers[b].not_equal(&self.registers[c])?;
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
                    self.pc = self.checked_jump(inst.sbx(), code.len())?;
                }
                RegOpCode::JumpIfTrue | RegOpCode::JumpIfFalse | RegOpCode::JumpIfSetLocal => {
                    let target = self.checked_jump(inst.sbx(), code.len())?;
                    let take = match op {
                        RegOpCode::JumpIfTrue => self.registers[a].as_bool()?,
                        RegOpCode::JumpIfFalse => !self.registers[a].as_bool()?,
                        _ => !matches!(self.registers[a], Value::Unset),
                    };
                    if take {
                        self.pc = target;
                    }
                }
                RegOpCode::GetGlobal => {
                    let name = self.constant_name(inst.bx() as usize)?;
                    self.registers[a] = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or(VmFault::UndefinedGlobal { name })?;
                }
                RegOpCode::SetGlobal => {
                    let name = self.constant_name(inst.bx() as usize)?;
                    self.globals.insert(name, self.registers[a].clone());
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
                            let val = d.borrow().get(&key).cloned().ok_or_else(|| {
                                VmFault::KeyNotFound {
                                    key: field_name.to_string(),
                                }
                            })?;
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
                            self.check_dict_insert(d, &key)?;
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
                            self.registers[a] = d
                                .borrow()
                                .get(k)
                                .cloned()
                                .ok_or_else(|| VmFault::KeyNotFound { key: k.to_string() })?;
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
                            self.registers[a] = Value::String(Rc::new(string_char_at(s, *i)?));
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
                            self.check_dict_insert(d, key)?;
                            d.borrow_mut().upsert(key.clone(), val);
                        }
                        Value::Bytes(bytes) => {
                            let Value::Int(index) = key else {
                                return Err(VmFault::TypeMismatch {
                                    expected: "Int index".into(),
                                    actual: key.type_name().into(),
                                });
                            };
                            let Value::Byte(byte) = val else {
                                return Err(VmFault::TypeMismatch {
                                    expected: "Byte".into(),
                                    actual: val.type_name().into(),
                                });
                            };
                            let mut bytes = bytes.borrow_mut();
                            let position = Self::position(bytes.len(), *index)?;
                            bytes[position] = byte;
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
                    let args = self
                        .registers
                        .get(a + 1..a + 1 + arg_count)
                        .ok_or_else(|| self.corrupted("call arguments exceed register file"))?
                        .to_vec();
                    match callee {
                        Value::Native(native) => {
                            Self::check_arity(native.arity, args.len())?;
                            let result = (native.func)(&args)?;
                            self.registers[a] = result;
                        }
                        Value::BoundMethod(bm) => match bm.kind {
                            crate::value::MethodKind::Native(func) => {
                                Self::check_arity(bm.arity, args.len())?;
                                let result = func(&bm.receiver, &args)?;
                                self.registers[a] = result;
                            }
                            _ => {
                                return Err(VmFault::NotCallable {
                                    type_name: format!("<method {}>", bm.name),
                                });
                            }
                        },
                        Value::Type(tag) => {
                            self.registers[a] = convert_via_type(tag, &args)?;
                        }
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
                            let result = self.invoke_index(func_idx, &args)?;
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
                            Self::check_arity(native.arity, 1)?;
                            let result = (native.func)(&[stream_arg])?;
                            self.registers[a] = result;
                        }
                        Value::BoundMethod(bm) => match bm.kind {
                            crate::value::MethodKind::Native(func) => {
                                Self::check_arity(bm.arity, 1)?;
                                let result = func(&bm.receiver, &[stream_arg])?;
                                self.registers[a] = result;
                            }
                            _ => {
                                return Err(VmFault::NotCallable {
                                    type_name: format!("<method {}>", bm.name),
                                });
                            }
                        },
                        Value::Type(tag) => {
                            self.registers[a] = convert_via_type(tag, &[stream_arg])?;
                        }
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
                            let result = self.invoke_index(func_idx, &[stream_arg])?;
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
                        self.check_function(func)?;
                        if !self.verified_module.get() {
                            aipo_bytecode::RegVerifier::verify_function(func, self.functions.len())
                                .map_err(|reason| self.corrupted(reason))?;
                        }
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
                    self.registers[a] = Value::List(Rc::new(RefCell::new((items).into())));
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
                        Value::Range(r) => Self::range_len(r.start, r.end)?,
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "collection, string or range".to_string(),
                                actual: other.type_name().to_string(),
                            });
                        }
                    };
                    self.registers[a] = Value::Int(check_safe_int(len)?);
                }
                RegOpCode::IterAt
                | RegOpCode::IterPrimary
                | RegOpCode::IterKey
                | RegOpCode::IterValue => {
                    let (index_reg, mode) = match op {
                        RegOpCode::IterPrimary => (c, 0),
                        RegOpCode::IterKey => (c, 1),
                        RegOpCode::IterValue => (c, 2),
                        _ => (c >> 2, c & 3),
                    };
                    if mode > 2 {
                        return Err(self.corrupted("invalid iteration projection"));
                    }
                    let Value::Int(ordinal) = self.registers[index_reg] else {
                        return Err(VmFault::TypeMismatch {
                            expected: "Int iteration index".into(),
                            actual: self.registers[index_reg].type_name().into(),
                        });
                    };
                    let projected = match &self.registers[b] {
                        Value::List(items) | Value::Set(items) => {
                            let items = items.borrow();
                            let position = Self::position(items.len(), ordinal)?;
                            if mode == 1 {
                                Value::Int(position as i64)
                            } else {
                                items[position].clone()
                            }
                        }
                        Value::Dict(dict) => {
                            let dict = dict.borrow();
                            let position = Self::position(dict.len(), ordinal)?;
                            let (key, value) = &dict.entries()[position];
                            if mode == 2 {
                                value.clone()
                            } else {
                                key.clone()
                            }
                        }
                        Value::Bytes(bytes) => {
                            let bytes = bytes.borrow();
                            let position = Self::position(bytes.len(), ordinal)?;
                            if mode == 1 {
                                Value::Int(position as i64)
                            } else {
                                Value::Byte(bytes[position])
                            }
                        }
                        Value::Range(range) => {
                            let length = usize::try_from(Self::range_len(range.start, range.end)?)
                                .map_err(|_| self.corrupted("range length exceeds target usize"))?;
                            let position = Self::position(length, ordinal)?;
                            let value = if mode == 1 {
                                position as i64
                            } else {
                                range
                                    .start
                                    .checked_add(position as i64)
                                    .ok_or_else(|| self.corrupted("range value overflow"))?
                            };
                            Value::Int(check_safe_int(value)?)
                        }
                        Value::String(text) => {
                            if mode == 1 {
                                Value::Int(Self::position(text.chars().count(), ordinal)? as i64)
                            } else {
                                Value::String(Rc::new(string_char_at(text, ordinal)?))
                            }
                        }
                        other => {
                            return Err(VmFault::TypeMismatch {
                                expected: "iterable collection".into(),
                                actual: other.type_name().into(),
                            });
                        }
                    };
                    self.registers[a] = projected;
                }
                RegOpCode::IterGuard => {
                    let value = self.registers[a].clone();
                    let length = Self::guarded_revision(&value);
                    self.iteration_guards.push((value, length));
                }
                RegOpCode::IterGuardEnd => {
                    if self.iteration_guards.len() <= guard_base {
                        return Err(self.corrupted("iteration guard underflow"));
                    }
                    self.iteration_guards.pop();
                }
                RegOpCode::AssertContract | RegOpCode::AssertContractNullable => {
                    let index = inst.bx() as usize;
                    let type_name = self.constant_name(index)?;
                    let position = self.constant_name(index + 1)?;
                    let nullable = op == RegOpCode::AssertContractNullable;
                    let value = &self.registers[a];
                    if !value.is_failure() && !self.contract_holds(&type_name, nullable, value) {
                        let actual = match value {
                            Value::Struct(instance) => instance.borrow().type_name.clone(),
                            other => other.type_name().into(),
                        };
                        return Err(VmFault::ContractViolation {
                            position,
                            expected: format!("{type_name}{}", if nullable { "?" } else { "" }),
                            actual,
                        });
                    }
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
                    let target = self.checked_jump(inst.sbx(), code.len())?;
                    self.handler_stack
                        .push((target, a as u8, self.iteration_guards.len()));
                }
                RegOpCode::PopHandler => {
                    self.handler_stack
                        .pop()
                        .ok_or_else(|| self.corrupted("failure handler underflow"))?;
                }
                RegOpCode::Fail => {
                    let val = &self.registers[a];
                    let (msg, payload) = match val {
                        Value::String(s) => ((**s).clone(), Value::None),
                        Value::Failure(f) => (f.message.clone(), f.payload.clone()),
                        other => (other.to_string(), other.clone()),
                    };
                    let failure = Value::failure_with_payload(msg, payload);
                    if let Some((handler_pc, err_reg, guards)) = self.handler_stack.pop() {
                        self.iteration_guards.truncate(guards);
                        self.registers[err_reg as usize] = failure;
                        self.pc = handler_pc;
                    } else {
                        return Ok(failure);
                    }
                }
                RegOpCode::PropagateFailure => {
                    if self.registers[a].is_failure() {
                        let failure = self.registers[a].clone();
                        if let Some((handler_pc, err_reg, guards)) = self.handler_stack.pop() {
                            self.iteration_guards.truncate(guards);
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

        let compiled = RegEmitter::new()
            .compile_function(&func)
            .expect("supported register IR");
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
        let list = Value::List(Rc::new(RefCell::new(
            (vec![Value::Int(10), Value::Int(20)]).into(),
        )));
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
        vm.functions = vec![Rc::new(adder)];
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
        vm.functions = vec![Rc::new(double)];

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
        vm.functions = vec![Rc::new(recurse)];
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
