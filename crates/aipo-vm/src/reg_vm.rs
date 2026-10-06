//! Virtual register execution engine for 32-bit instructions (Marco 3 / ADP-014).

use crate::arena::ArenaAllocator;
use crate::fault::VmFault;
use crate::value::{Value, check_safe_int};
use aipo_bytecode::{RegInstruction, RegOpCode};
use std::rc::Rc;

/// Compact virtual register machine for embedded execution and fast scripts.
#[derive(Debug)]
pub struct RegVm {
    /// 256 virtual registers (fixed array of 16-byte Values = 4 KiB total).
    pub registers: [Value; 256],
    /// Constant pool.
    pub constants: Vec<Value>,
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
            arena: ArenaAllocator::with_default_capacity(),
            pc: 0,
        }
    }

    /// Executes a compiled register function.
    ///
    /// # Errors
    /// Returns [`VmFault`] if a runtime error occurs.
    pub fn run_function(
        &mut self,
        func: &aipo_bytecode::RegCompiledFunction,
    ) -> Result<Value, VmFault> {
        self.constants = func
            .constants
            .iter()
            .map(|c| match c {
                aipo_bytecode::Constant::Nil => Value::None,
                aipo_bytecode::Constant::Bool(b) => Value::Bool(*b),
                aipo_bytecode::Constant::Int(n) => Value::Int(*n),
                aipo_bytecode::Constant::Float(f) => Value::Float(*f),
                aipo_bytecode::Constant::String(s) => Value::String(Rc::new(s.clone())),
            })
            .collect();
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
                RegOpCode::Return => {
                    return Ok(self.registers[a].clone());
                }
                _ => {}
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
}
