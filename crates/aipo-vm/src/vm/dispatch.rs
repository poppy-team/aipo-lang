//! Instruction dispatch for the Aipo stack machine: the `step` loop,
//! operand readers, and every opcode arm. Pure helpers live in `helpers`.
use super::Vm;
use super::helpers::{collection_identity, length_of, normalize_range, value_diagnostic_name};
use crate::fault::{VmError, VmFault};
use crate::frame::HandlerFrame;
use crate::value::{
    DictMap, FailureValue, MethodKind, StructInstance, Value, check_finite_float, check_safe_int,
};
use aipo_bytecode::BytecodeModule;
use aipo_bytecode::opcode::{Constant, OpCode};
use byteorder::{BigEndian, ByteOrder};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
impl Vm {
    /// Executes a single instruction at current `ip`.
    ///
    /// Returns `Ok(true)` if execution completed or top-level return executed.
    ///
    /// # Errors
    /// Returns `VmError` on runtime fault or invalid instruction.
    pub fn step(&mut self, module: &BytecodeModule) -> Result<bool, VmError> {
        if self.ip >= module.code.len() || self.halted_with.is_some() {
            return Ok(true);
        }

        let opcode_byte = module.code[self.ip];
        self.ip += 1;

        let opcode = OpCode::try_from(opcode_byte).map_err(|b| VmFault::CorruptedBytecode {
            offset: self.ip - 1,
            reason: format!("unknown opcode 0x{b:02x}"),
        })?;

        match opcode {
            OpCode::Constant => {
                let idx = self.read_u16(module)? as usize;
                let c = module
                    .constants
                    .get(idx)
                    .ok_or_else(|| VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("constant index {idx} out of bounds"),
                    })?;

                let val = match c {
                    Constant::Nil => Value::None,
                    Constant::Bool(b) => Value::Bool(*b),
                    Constant::Int(n) => Value::Int(check_safe_int(*n)?),
                    Constant::Float(f) => Value::Float(check_finite_float(*f)?),
                    Constant::String(s) => Value::String(Rc::new(s.clone())),
                };
                self.push(val)?;
            }
            OpCode::Nil => self.push(Value::None)?,
            OpCode::True => self.push(Value::Bool(true))?,
            OpCode::False => self.push(Value::Bool(false))?,
            OpCode::Unset => self.push(Value::Unset)?,
            OpCode::JumpIfSetLocal => {
                let slot = self.read_u16(module)? as usize;
                let rel = self.read_i16(module)? as isize;
                let base = self.frames.last().map_or(0, |f| f.stack_base);
                // A slot holding anything other than the sentinel was supplied by the caller,
                // so its default must not be evaluated; `none` counts as supplied.
                let is_set = !matches!(self.stack.get(base + slot), Some(Value::Unset));
                if is_set {
                    let target = (self.ip as isize) + rel;
                    if target < 0 || (target as usize) > module.code.len() {
                        return Err(VmFault::CorruptedBytecode {
                            offset: self.ip - 2,
                            reason: format!("jump target {target} out of range"),
                        }
                        .into());
                    }
                    self.ip = target as usize;
                }
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let v = self.peek()?.clone();
                self.push(v)?;
            }
            OpCode::GetLocal => {
                let slot = self.read_u16(module)? as usize;
                let base = self.frames.last().map_or(0, |f| f.stack_base);
                let val = self
                    .stack
                    .get(base + slot)
                    .cloned()
                    .ok_or(VmFault::StackUnderflow)?;
                self.push(val)?;
            }
            OpCode::SetLocal => {
                let slot = self.read_u16(module)? as usize;
                let base = self.frames.last().map_or(0, |f| f.stack_base);
                let val = self.pop()?;
                if base + slot >= self.stack.len() {
                    return Err(VmFault::StackUnderflow.into());
                }
                self.stack[base + slot] = val;
            }
            OpCode::GetGlobal => {
                let idx = self.read_u16(module)? as usize;
                let name = module
                    .names
                    .get(idx)
                    .ok_or_else(|| VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("name index {idx} out of bounds"),
                    })?;
                let val = self
                    .globals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| VmFault::UndefinedGlobal { name: name.clone() })?;
                self.push(val)?;
            }
            OpCode::SetGlobal => {
                let idx = self.read_u16(module)? as usize;
                let name = module
                    .names
                    .get(idx)
                    .ok_or_else(|| VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("name index {idx} out of bounds"),
                    })?;
                let val = self.pop()?;
                self.globals.insert(name.clone(), val);
            }
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.add(&b)?;
                self.push(res)?;
            }
            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.sub(&b)?;
                self.push(res)?;
            }
            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.mul(&b)?;
                self.push(res)?;
            }
            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.div(&b)?;
                self.push(res)?;
            }
            OpCode::IntDiv => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.int_div(&b)?;
                self.push(res)?;
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.modulo(&b)?;
                self.push(res)?;
            }
            OpCode::Neg => {
                let a = self.pop()?;
                let res = a.negate()?;
                self.push(res)?;
            }
            OpCode::Not => {
                let a = self.pop()?;
                let res = a.not()?;
                self.push(res)?;
            }
            OpCode::Equal => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.equal(&b)?;
                self.push(res)?;
            }
            OpCode::NotEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.not_equal(&b)?;
                self.push(res)?;
            }
            OpCode::Less => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.less(&b)?;
                self.push(res)?;
            }
            OpCode::LessEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.less_equal(&b)?;
                self.push(res)?;
            }
            OpCode::Greater => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.greater(&b)?;
                self.push(res)?;
            }
            OpCode::GreaterEqual => {
                let b = self.pop()?;
                let a = self.pop()?;
                let res = a.greater_equal(&b)?;
                self.push(res)?;
            }
            OpCode::Jump => {
                let rel = self.read_i16(module)? as isize;
                let target = (self.ip as isize) + rel;
                if target < 0 || (target as usize) > module.code.len() {
                    return Err(VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("jump target {target} out of range"),
                    }
                    .into());
                }
                self.ip = target as usize;
            }
            OpCode::JumpIfFalse => {
                let rel = self.read_i16(module)? as isize;
                let cond = self.pop()?;
                let is_true = cond.as_bool()?;
                if !is_true {
                    let target = (self.ip as isize) + rel;
                    if target < 0 || (target as usize) > module.code.len() {
                        return Err(VmFault::CorruptedBytecode {
                            offset: self.ip - 2,
                            reason: format!("jump target {target} out of range"),
                        }
                        .into());
                    }
                    self.ip = target as usize;
                }
            }
            OpCode::Call => {
                let arg_count = self.read_u8(module)? as usize;
                self.begin_call(module, arg_count)?;
            }
            OpCode::Return => {
                let ret_val = if self.stack.is_empty() {
                    Value::None
                } else {
                    self.pop()?
                };

                if let Some(frame) = self.frames.pop() {
                    self.upvalue_frames.pop();
                    self.stack.truncate(frame.result_base());
                    self.push(ret_val)?;
                    self.ip = frame.return_ip;
                    // A returning frame releases its provisional mutations: the boundary
                    // marker already verified or rolled them back.
                    self.mutation_journal.truncate(frame.journal_start);
                } else {
                    self.push(ret_val)?;
                    return Ok(true);
                }
            }
            OpCode::GetField => {
                let name_idx = self.read_u16(module)? as usize;
                let field_name =
                    module
                        .names
                        .get(name_idx)
                        .ok_or_else(|| VmFault::CorruptedBytecode {
                            offset: self.ip - 2,
                            reason: format!("name index {name_idx} out of bounds"),
                        })?;

                let target = self.pop()?;
                if let Value::Failure(err) = &target {
                    if field_name == "message" {
                        self.push(Value::String(Rc::new(err.message.clone())))?;
                        return Ok(false);
                    }
                    self.push(target)?;
                    return Ok(false);
                }

                match &target {
                    Value::Struct(inst) => {
                        let field = inst.borrow().get_field(field_name).cloned();
                        if let Some(val) = field {
                            self.push(val)?;
                        } else {
                            let type_name = inst.borrow().type_name.clone();
                            if let Some((entry_ip, total_arity)) = self
                                .struct_methods
                                .get(&(type_name.clone(), field_name.clone()))
                            {
                                let entry_ip = *entry_ip;
                                let total_arity = *total_arity;
                                self.push(Value::BoundMethod {
                                    name: format!("{type_name}.{field_name}"),
                                    arity: total_arity.saturating_sub(1),
                                    receiver: Box::new(target.clone()),
                                    kind: MethodKind::Function {
                                        entry_ip,
                                        total_arity,
                                    },
                                })?;
                            } else {
                                return Err(VmFault::NoSuchField {
                                    type_name,
                                    field: field_name.clone(),
                                }
                                .into());
                            }
                        }
                    }
                    Value::Dict(d) => {
                        let key = Value::String(Rc::new(field_name.clone()));
                        let found = d.borrow().get(&key).cloned();
                        if let Some(v) = found {
                            self.push(v)?;
                        } else if let Some(method) = self.bind_method(&target, field_name) {
                            self.push(method)?;
                        } else {
                            return Err(VmFault::KeyNotFound {
                                key: field_name.clone(),
                            }
                            .into());
                        }
                    }
                    other => {
                        if let Some(method) = self.bind_method(other, field_name) {
                            self.push(method)?;
                        } else {
                            return Err(VmFault::TypeMismatch {
                                expected: "struct, collection, or type with that member"
                                    .to_string(),
                                actual: format!("{}.{field_name}", other.type_name()),
                            }
                            .into());
                        }
                    }
                }
            }
            OpCode::SetField => {
                let name_idx = self.read_u16(module)? as usize;
                let field_name =
                    module
                        .names
                        .get(name_idx)
                        .ok_or_else(|| VmFault::CorruptedBytecode {
                            offset: self.ip - 2,
                            reason: format!("name index {name_idx} out of bounds"),
                        })?;

                let new_val = self.pop()?;
                let target = self.pop()?;

                if let Value::Struct(inst) = &target {
                    let type_name = inst.borrow().type_name.clone();
                    // Canon applies a guarded update provisionally and verifies it at the end
                    // of the enclosing mutable operation, so the frame-entry value is kept for
                    // a possible rollback instead of validating the assignment right here.
                    let entry_value = inst.borrow().get_field(field_name).cloned();
                    let published = !inst.borrow().under_construction;
                    inst.borrow_mut().set_field(field_name, new_val)?;

                    if published && self.type_is_guarded(&type_name) {
                        if let Some(previous) = entry_value {
                            self.journal_mutation(Rc::clone(inst), field_name.clone(), previous);
                        }
                    }
                } else {
                    return Err(VmFault::TypeMismatch {
                        expected: "struct instance".to_string(),
                        actual: target.type_name().to_string(),
                    }
                    .into());
                }
            }
            OpCode::GetIndex => {
                let index = self.pop()?;
                let target = self.pop()?;

                if target.is_failure() {
                    self.push(target)?;
                    return Ok(false);
                }
                if index.is_failure() {
                    self.push(index)?;
                    return Ok(false);
                }

                match (&target, &index) {
                    (Value::List(l), Value::Range { start, end }) => {
                        let items = l.borrow().clone();
                        let (from, to) = normalize_range(*start, *end, items.len());
                        let slice: Vec<Value> = items[from..to].to_vec();
                        self.push(Value::List(Rc::new(RefCell::new(slice))))?;
                    }
                    (Value::String(s), Value::Range { start, end }) => {
                        let chars: Vec<char> = s.chars().collect();
                        let (from, to) = normalize_range(*start, *end, chars.len());
                        let slice: String = chars[from..to].iter().collect();
                        self.push(Value::String(Rc::new(slice)))?;
                    }
                    (Value::Bytes(b), Value::Range { start, end }) => {
                        let (from, to) = normalize_range(*start, *end, b.len());
                        self.push(Value::Bytes(Rc::new(b[from..to].to_vec())))?;
                    }
                    (Value::Range { start, end }, Value::Int(i)) => {
                        // Positional access into a half-open range, which is what makes
                        // `each i in 0..n` (documented canon) lower to the same
                        // index-based loop every other iterable uses.
                        let len = usize::try_from((*end - *start).max(0)).unwrap_or(0);
                        #[allow(clippy::cast_possible_wrap)]
                        let actual = if *i < 0 { (len as i64) + *i } else { *i };
                        let resolved = usize::try_from(actual)
                            .ok()
                            .filter(|index| *index < len)
                            .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                        #[allow(clippy::cast_possible_wrap)]
                        let offset = resolved as i64;
                        self.push(Value::Int(start + offset))?;
                    }
                    (Value::Bytes(b), Value::Int(i)) => {
                        let len = b.len();
                        #[allow(clippy::cast_possible_wrap)]
                        let actual_idx = if *i < 0 { (len as i64) + *i } else { *i };
                        let idx_usize = usize::try_from(actual_idx)
                            .map_err(|_| VmFault::IndexOutOfRange { index: *i, len })?;
                        let byte = *b
                            .get(idx_usize)
                            .ok_or(VmFault::IndexOutOfRange { index: *i, len })?;
                        self.push(Value::Byte(byte))?;
                    }
                    (Value::List(l), Value::Int(i)) => {
                        let len = l.borrow().len();
                        #[allow(clippy::cast_possible_wrap)]
                        let actual_idx = if *i < 0 { (len as i64) + *i } else { *i };
                        let idx_usize = usize::try_from(actual_idx)
                            .map_err(|_| VmFault::IndexOutOfRange { index: *i, len })?;
                        if idx_usize >= len {
                            return Err(VmFault::IndexOutOfRange { index: *i, len }.into());
                        }
                        let val = l.borrow()[idx_usize].clone();
                        self.push(val)?;
                    }
                    (Value::Dict(d), _) => {
                        if let Some(v) = d.borrow().get(&index) {
                            self.push(v.clone())?;
                        } else {
                            return Err(VmFault::KeyNotFound {
                                key: format!("{index}"),
                            }
                            .into());
                        }
                    }
                    (Value::String(s), Value::Int(i)) => {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len();
                        #[allow(clippy::cast_possible_wrap)]
                        let actual_idx = if *i < 0 { (len as i64) + *i } else { *i };
                        let idx_usize = usize::try_from(actual_idx)
                            .map_err(|_| VmFault::IndexOutOfRange { index: *i, len })?;
                        if idx_usize >= len {
                            return Err(VmFault::IndexOutOfRange { index: *i, len }.into());
                        }
                        let ch = chars[idx_usize];
                        self.push(Value::String(Rc::new(ch.to_string())))?;
                    }
                    _ => {
                        return Err(VmFault::TypeMismatch {
                            expected: "indexable collection or string".to_string(),
                            actual: format!(
                                "{} indexed by {}",
                                target.type_name(),
                                index.type_name()
                            ),
                        }
                        .into());
                    }
                }
            }
            OpCode::SetIndex => {
                let new_val = self.pop()?;
                let index = self.pop()?;
                let target = self.pop()?;

                if target.is_failure() {
                    self.push(target)?;
                    return Ok(false);
                }
                if index.is_failure() {
                    self.push(index)?;
                    return Ok(false);
                }
                if new_val.is_failure() {
                    self.push(new_val)?;
                    return Ok(false);
                }

                match (&target, &index) {
                    (Value::List(l), Value::Int(i)) => {
                        let len = l.borrow().len();
                        #[allow(clippy::cast_possible_wrap)]
                        let actual_idx = if *i < 0 { (len as i64) + *i } else { *i };
                        let idx_usize = usize::try_from(actual_idx)
                            .map_err(|_| VmFault::IndexOutOfRange { index: *i, len })?;
                        if idx_usize >= len {
                            return Err(VmFault::IndexOutOfRange { index: *i, len }.into());
                        }
                        l.borrow_mut()[idx_usize] = new_val;
                    }
                    (Value::Dict(d), _) => {
                        d.borrow_mut().upsert(index, new_val);
                    }
                    _ => {
                        return Err(VmFault::TypeMismatch {
                            expected: "mutable collection".to_string(),
                            actual: target.type_name().to_string(),
                        }
                        .into());
                    }
                }
            }
            OpCode::BuildList => {
                let count = self.read_u16(module)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(Value::List(Rc::new(RefCell::new(items))))?;
            }
            OpCode::BuildDict => {
                let count = self.read_u16(module)? as usize;
                let mut pairs = Vec::with_capacity(count);
                for _ in 0..count {
                    let v = self.pop()?;
                    let k = self.pop()?;
                    pairs.push((k, v));
                }
                pairs.reverse();
                self.push(Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(
                    pairs,
                )))))?;
            }
            OpCode::BuildStruct => {
                let type_idx = self.read_u16(module)? as usize;
                let field_count = self.read_u16(module)? as usize;
                let defer_fixed = self.read_u8(module)? != 0;
                let type_name =
                    module
                        .names
                        .get(type_idx)
                        .ok_or_else(|| VmFault::CorruptedBytecode {
                            offset: self.ip - 5,
                            reason: format!("type name index {type_idx} out of bounds"),
                        })?;

                let mut values = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    values.push(self.pop()?);
                }
                values.reverse();

                let instance = if let Some(defs) = self.struct_defs.get(type_name) {
                    let mut fields = Vec::new();
                    let mut fixed_fields = HashSet::new();
                    for ((f_name, is_fixed), val) in defs.iter().cloned().zip(values) {
                        // A deferred construction keeps `fixed` fields mutable until `init`
                        // finishes and the instance is sealed.
                        if is_fixed && !defer_fixed {
                            fixed_fields.insert(f_name.clone());
                        }
                        fields.push((f_name, val));
                    }
                    StructInstance {
                        type_name: type_name.clone(),
                        fields,
                        fixed_fields,
                        under_construction: defer_fixed,
                    }
                } else {
                    let fields = values
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| (format!("field_{i}"), v))
                        .collect();
                    StructInstance {
                        type_name: type_name.clone(),
                        fields,
                        fixed_fields: HashSet::new(),
                        under_construction: false,
                    }
                };

                // While `init` still has to run, the invariant is evaluated at seal time
                // instead, so it observes the fields `init` assigned.
                if !defer_fixed {
                    if let Some(validator) = self.struct_invariants.get(type_name) {
                        validator(&instance).map_err(|msg| VmFault::InvariantViolation {
                            type_name: type_name.clone(),
                            message: msg,
                        })?;
                    }
                }

                self.push(Value::Struct(Rc::new(RefCell::new(instance))))?;
            }
            OpCode::SealStruct => {
                let value = self.pop()?;
                let instance = match &value {
                    Value::Struct(instance) => Rc::clone(instance),
                    other => {
                        let actual = other.type_name().to_string();
                        self.push(value)?;
                        return Err(VmFault::TypeMismatch {
                            expected: "Struct under construction for SealStruct".to_string(),
                            actual,
                        }
                        .into());
                    }
                };
                let type_name = instance.borrow().type_name.clone();
                let fixed_fields: HashSet<String> = self
                    .struct_defs
                    .get(&type_name)
                    .map(|defs| {
                        defs.iter()
                            .filter(|(_, is_fixed)| *is_fixed)
                            .map(|(name, _)| name.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                {
                    let mut borrowed = instance.borrow_mut();
                    borrowed.seal(fixed_fields);
                    if let Some(validator) = self.struct_invariants.get(&type_name) {
                        validator(&borrowed).map_err(|msg| VmFault::InvariantViolation {
                            type_name: type_name.clone(),
                            message: msg,
                        })?;
                    }
                }

                self.push(Value::Struct(instance))?;
            }
            OpCode::AssertInvariant => {
                let name_idx = self.read_u16(module)? as usize;
                let type_name = module.names.get(name_idx).cloned().ok_or_else(|| {
                    VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("type name index {name_idx} out of bounds"),
                    }
                })?;
                let result = self.pop()?;
                if !matches!(result, Value::Bool(true)) {
                    return Err(VmFault::InvariantViolation {
                        type_name,
                        message: "invariant() did not hold after construction".to_string(),
                    }
                    .into());
                }
            }
            OpCode::AssertContract => {
                let type_idx = self.read_u16(module)? as usize;
                let nullable = self.read_u8(module)? != 0;
                let position_idx = self.read_u16(module)? as usize;
                let type_name = module.names.get(type_idx).cloned().ok_or_else(|| {
                    VmFault::CorruptedBytecode {
                        offset: self.ip - 5,
                        reason: format!("contract type index {type_idx} out of bounds"),
                    }
                })?;
                let position = module.names.get(position_idx).cloned().ok_or_else(|| {
                    VmFault::CorruptedBytecode {
                        offset: self.ip - 5,
                        reason: format!("contract position index {position_idx} out of bounds"),
                    }
                })?;
                let operation_count = self.read_u8(module)? as usize;
                let mut operations = Vec::with_capacity(operation_count);
                for _ in 0..operation_count {
                    let name_idx = self.read_u16(module)? as usize;
                    let arity = self.read_u8(module)? as usize;
                    let name = module.names.get(name_idx).cloned().ok_or_else(|| {
                        VmFault::CorruptedBytecode {
                            offset: self.ip - 3,
                            reason: format!("contract operation index {name_idx} out of bounds"),
                        }
                    })?;
                    operations.push((name, arity));
                }
                // The contract inspects the value in place so the same instruction serves a
                // parameter check (after `Load`) and a return check (before `Return`).
                let value = self.stack.last().cloned().ok_or(VmFault::StackUnderflow)?;
                // An omitted argument keeps the sentinel (reported as a missing argument, not a
                // type violation) and `fail(...)` ends the path without satisfying the return
                // contract, so neither is a contract violation.
                let skippable = matches!(value, Value::Unset | Value::Failure(_));
                if !skippable && !self.contract_holds(&type_name, nullable, &value) {
                    return Err(VmFault::ContractViolation {
                        position,
                        expected: if nullable {
                            format!("{type_name}?")
                        } else {
                            type_name
                        },
                        actual: value_diagnostic_name(&value),
                    }
                    .into());
                }
                // Canon makes an interface structural, so the contract asks whether the value
                // exposes the operations the interface declares. `none` satisfies `T?` without
                // exposing anything, which is exactly what `T?` means.
                if !skippable && !matches!(value, Value::None) {
                    for (operation, declared) in &operations {
                        let actual = self.operation_arity(&value, operation);
                        if actual != Some(*declared) {
                            // The contract names the *caller-visible* arity (the receiver is not
                            // an argument at the call site), which is also how native method
                            // arities are declared.
                            let required = format!("{type_name}.{operation}/{declared}");
                            let actual_name = value_diagnostic_name(&value);
                            let found = match actual {
                                Some(arity) => format!("{actual_name}.{operation}/{arity}"),
                                None => format!("{actual_name} without '{operation}'"),
                            };
                            return Err(VmFault::ContractViolation {
                                position,
                                expected: required,
                                actual: found,
                            }
                            .into());
                        }
                    }
                }
            }
            OpCode::CheckMutations => {
                let base = self.frames.last().map_or(0, |frame| frame.journal_start);
                self.commit_mutations(module, base)?;
            }
            OpCode::MakeFunction => {
                let fn_idx = self.read_u16(module)? as usize;
                let info =
                    module
                        .functions
                        .get(fn_idx)
                        .ok_or_else(|| VmFault::CorruptedBytecode {
                            offset: self.ip - 2,
                            reason: format!("function index {fn_idx} out of bounds"),
                        })?;
                let entry_ip = info.entry_ip;
                let arity = info.params;
                self.push(Value::Function { entry_ip, arity })?;
            }
            OpCode::MakeClosure => {
                let fn_idx = self.read_u16(module)? as usize;
                let upvalue_count = self.read_u16(module)? as usize;
                let info =
                    module
                        .functions
                        .get(fn_idx)
                        .ok_or_else(|| VmFault::CorruptedBytecode {
                            offset: self.ip - 4,
                            reason: format!("function index {fn_idx} out of bounds"),
                        })?;
                let entry_ip = info.entry_ip;
                let arity = info.params;
                let mut upvalues = Vec::with_capacity(upvalue_count);
                for _ in 0..upvalue_count {
                    upvalues.push(Rc::new(RefCell::new(self.pop()?)));
                }
                upvalues.reverse();
                self.push(Value::Closure {
                    entry_ip,
                    arity,
                    upvalues,
                })?;
            }
            OpCode::GetUpvalue => {
                let idx = self.read_u16(module)? as usize;
                let frame = self.upvalue_frames.last().cloned().flatten();
                let cell = frame
                    .and_then(|cells| cells.get(idx).cloned())
                    .ok_or(VmFault::StackUnderflow)?;
                let val = cell.borrow().clone();
                self.push(val)?;
            }
            OpCode::SetUpvalue => {
                let idx = self.read_u16(module)? as usize;
                let value = self.pop()?;
                let frame = self.upvalue_frames.last().cloned().flatten();
                let cell = frame
                    .and_then(|cells| cells.get(idx).cloned())
                    .ok_or(VmFault::StackUnderflow)?;
                *cell.borrow_mut() = value;
            }
            OpCode::Range => {
                let end = self.pop()?;
                let start = self.pop()?;
                match (start, end) {
                    (Value::Int(s), Value::Int(e)) => {
                        self.push(Value::Range { start: s, end: e })?
                    }
                    (a, b) => {
                        return Err(VmFault::TypeMismatch {
                            expected: "Int range bounds".to_string(),
                            actual: format!("{} and {}", a.type_name(), b.type_name()),
                        }
                        .into());
                    }
                }
            }
            OpCode::Len => {
                let value = self.pop()?;
                if value.is_failure() {
                    self.push(value)?;
                } else {
                    let len = length_of(&value)?;
                    self.push(len)?;
                }
            }
            OpCode::TypeIs => {
                let tag = self.pop()?;
                let value = self.pop()?;
                match tag {
                    Value::Type(tag) => self.push(Value::Bool(tag.matches(&value)))?,
                    other => {
                        return Err(VmFault::TypeMismatch {
                            expected: "type value on the right of `is`".to_string(),
                            actual: other.type_name().to_string(),
                        }
                        .into());
                    }
                }
            }
            OpCode::IterGuard => {
                let value = self.pop()?;
                if let Some(id) = collection_identity(&value) {
                    self.active_iterations.push(id);
                }
            }
            OpCode::IterGuardEnd => {
                self.active_iterations.pop();
            }
            OpCode::Fail => {
                let err_val = self.pop()?;
                let message = match err_val {
                    Value::String(s) => (*s).clone(),
                    Value::Failure(f) => f.message.clone(),
                    other => other.to_string(),
                };
                let failure = Value::Failure(Rc::new(FailureValue { message }));
                self.handle_failure(failure)?;
            }
            OpCode::FillSelfCapture => {
                let _name_idx = self.read_u16(module)? as usize;
                let closure = self.pop()?;
                // The builder reserves exactly one `Unset` placeholder — the self capture —
                // and no other capture can hold the sentinel at creation time, because
                // captures snapshot already-initialized bindings.
                let cells = match &closure {
                    Value::Closure { upvalues, .. } => upvalues.clone(),
                    other => {
                        return Err(VmFault::TypeMismatch {
                            expected: "closure".to_string(),
                            actual: other.type_name().to_string(),
                        }
                        .into());
                    }
                };
                let filled = cells
                    .iter()
                    .find(|cell| matches!(&*cell.borrow(), Value::Unset))
                    .ok_or_else(|| VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: "FillSelfCapture without an Unset placeholder cell".to_string(),
                    })?;
                *filled.borrow_mut() = closure.clone();
                self.push(closure)?;
            }
            OpCode::CheckFailure => {
                // Statement boundary: an unconsumed recoverable Failure ends this path.
                if self.stack.last().is_some_and(Value::is_failure) {
                    let failure = self.pop()?;
                    self.handle_failure(failure)?;
                }
            }
            OpCode::OrElse => {
                let right = self.pop()?;
                let left = self.pop()?;
                if left.is_failure() {
                    self.push(right)?;
                } else {
                    self.push(left)?;
                }
            }
            OpCode::PushHandler => {
                let rel = self.read_i16(module)? as isize;
                let target = (self.ip as isize) + rel;
                if target < 0 || (target as usize) > module.code.len() {
                    return Err(VmFault::CorruptedBytecode {
                        offset: self.ip - 2,
                        reason: format!("PushHandler target {target} out of range"),
                    }
                    .into());
                }
                self.handlers.push(HandlerFrame::new(
                    target as usize,
                    self.stack.len(),
                    self.frames.len(),
                ));
            }
            OpCode::PopHandler => {
                self.handlers.pop();
            }
        }

        Ok(false)
    }

    pub(super) fn read_u8(&mut self, module: &BytecodeModule) -> Result<u8, VmFault> {
        if self.ip >= module.code.len() {
            return Err(VmFault::CorruptedBytecode {
                offset: self.ip,
                reason: "unexpected end of bytecode".to_string(),
            });
        }
        let b = module.code[self.ip];
        self.ip += 1;
        Ok(b)
    }

    pub(super) fn read_u16(&mut self, module: &BytecodeModule) -> Result<u16, VmFault> {
        if self.ip + 2 > module.code.len() {
            return Err(VmFault::CorruptedBytecode {
                offset: self.ip,
                reason: "truncated 16-bit operand".to_string(),
            });
        }
        let val = BigEndian::read_u16(&module.code[self.ip..self.ip + 2]);
        self.ip += 2;
        Ok(val)
    }

    pub(super) fn read_i16(&mut self, module: &BytecodeModule) -> Result<i16, VmFault> {
        if self.ip + 2 > module.code.len() {
            return Err(VmFault::CorruptedBytecode {
                offset: self.ip,
                reason: "truncated 16-bit jump offset".to_string(),
            });
        }
        let val = BigEndian::read_i16(&module.code[self.ip..self.ip + 2]);
        self.ip += 2;
        Ok(val)
    }
}
