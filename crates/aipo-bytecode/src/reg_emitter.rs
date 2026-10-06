//! Emitter converting Core IR into 32-bit virtual register instructions (Marco 3 / ADP-014).

use crate::instruction::{RegInstruction, RegOpCode};
use crate::opcode::Constant;
use aipo_ir::{BinaryOp, CoreConstant, CoreFunction, CoreInst, UnaryOp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A compiled function for the virtual register machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegCompiledFunction {
    /// Function name.
    pub name: String,
    /// 32-bit register bytecode instructions.
    pub instructions: Vec<RegInstruction>,
    /// Constant pool.
    pub constants: Vec<Constant>,
    /// Number of virtual registers required.
    pub num_registers: usize,
}

/// Emitter for converting `CoreFunction` into `RegCompiledFunction`.
#[derive(Debug, Default)]
pub struct RegEmitter {
    constants: Vec<Constant>,
    instructions: Vec<RegInstruction>,
    max_reg: usize,
}

impl RegEmitter {
    /// Creates a new register emitter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Compiles a Core IR function into a register function.
    pub fn compile_function(mut self, func: &CoreFunction) -> RegCompiledFunction {
        let mut slot_map = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            slot_map.insert(param.clone(), i);
        }
        for (i, local) in func.locals.iter().enumerate() {
            slot_map.insert(local.clone(), func.params.len() + i);
        }

        let temp_base = func.params.len() + func.locals.len();
        let mut top = temp_base;
        self.max_reg = temp_base;

        let mut core_to_reg: Vec<usize> = Vec::with_capacity(func.instructions.len());
        let mut jump_patches: Vec<(usize, isize, bool, u8)> = Vec::new();

        for inst in &func.instructions {
            core_to_reg.push(self.instructions.len());

            match inst {
                CoreInst::Constant(c, _) => {
                    self.track_reg(top);
                    match c {
                        CoreConstant::None => {
                            self.emit(RegInstruction::encode_abc(
                                RegOpCode::LoadNil,
                                top as u8,
                                0,
                                0,
                            ));
                        }
                        CoreConstant::Bool(b) => {
                            let val = if *b { 1 } else { 0 };
                            self.emit(RegInstruction::encode_abc(
                                RegOpCode::LoadBool,
                                top as u8,
                                val,
                                0,
                            ));
                        }
                        CoreConstant::Int(n) => {
                            if *n >= -65536 && *n <= 65535 {
                                self.emit(RegInstruction::encode_asbx(
                                    RegOpCode::LoadInt,
                                    top as u8,
                                    *n as i32,
                                ));
                            } else {
                                let c_idx = self.add_const(Constant::Int(*n));
                                self.emit(RegInstruction::encode_abx(
                                    RegOpCode::LoadConst,
                                    top as u8,
                                    c_idx as u32,
                                ));
                            }
                        }
                        CoreConstant::Float(f) => {
                            let c_idx = self.add_const(Constant::Float(*f));
                            self.emit(RegInstruction::encode_abx(
                                RegOpCode::LoadConst,
                                top as u8,
                                c_idx as u32,
                            ));
                        }
                        CoreConstant::String(s) => {
                            let c_idx = self.add_const(Constant::String(s.clone()));
                            self.emit(RegInstruction::encode_abx(
                                RegOpCode::LoadConst,
                                top as u8,
                                c_idx as u32,
                            ));
                        }
                    }
                    top += 1;
                }
                CoreInst::Load(name, _) => {
                    self.track_reg(top);
                    if let Some(&slot) = slot_map.get(name) {
                        self.emit(RegInstruction::encode_abc(
                            RegOpCode::Move,
                            top as u8,
                            slot as u16,
                            0,
                        ));
                    } else {
                        let c_idx = self.add_const(Constant::String(name.clone()));
                        self.emit(RegInstruction::encode_abx(
                            RegOpCode::GetGlobal,
                            top as u8,
                            c_idx as u32,
                        ));
                    }
                    top += 1;
                }
                CoreInst::Store(name, _) => {
                    if top > temp_base {
                        top -= 1;
                    }
                    if let Some(&slot) = slot_map.get(name) {
                        self.emit(RegInstruction::encode_abc(
                            RegOpCode::Move,
                            slot as u8,
                            top as u16,
                            0,
                        ));
                    } else {
                        let c_idx = self.add_const(Constant::String(name.clone()));
                        self.emit(RegInstruction::encode_abx(
                            RegOpCode::SetGlobal,
                            top as u8,
                            c_idx as u32,
                        ));
                    }
                }
                CoreInst::Binary(op, _) => {
                    if top >= 2 {
                        top -= 2;
                        let left = top;
                        let right = top + 1;
                        let reg_op = match op {
                            BinaryOp::Add => RegOpCode::Add,
                            BinaryOp::Sub => RegOpCode::Sub,
                            BinaryOp::Mul => RegOpCode::Mul,
                            BinaryOp::Div => RegOpCode::Div,
                            BinaryOp::IntDiv => RegOpCode::IntDiv,
                            BinaryOp::Mod => RegOpCode::Mod,
                            BinaryOp::Equal => RegOpCode::Equal,
                            BinaryOp::NotEqual => RegOpCode::NotEqual,
                            BinaryOp::Less => RegOpCode::Less,
                            BinaryOp::LessEqual => RegOpCode::LessEqual,
                            BinaryOp::Greater => RegOpCode::Greater,
                            BinaryOp::GreaterEqual => RegOpCode::GreaterEqual,
                            _ => RegOpCode::Equal,
                        };
                        self.emit(RegInstruction::encode_abc(
                            reg_op,
                            top as u8,
                            left as u16,
                            right as u8,
                        ));
                        top += 1;
                    }
                }
                CoreInst::Unary(op, _) => {
                    if top > 0 {
                        let arg = top - 1;
                        match op {
                            UnaryOp::Neg => {
                                self.emit(RegInstruction::encode_abc(
                                    RegOpCode::Neg,
                                    arg as u8,
                                    arg as u16,
                                    0,
                                ));
                            }
                            UnaryOp::Not => {
                                self.emit(RegInstruction::encode_abc(
                                    RegOpCode::Not,
                                    arg as u8,
                                    arg as u16,
                                    0,
                                ));
                            }
                            UnaryOp::Pos => {}
                        }
                    }
                }
                CoreInst::Jump(target, _) => {
                    let patch_pos = self.instructions.len();
                    self.emit(RegInstruction::encode_asbx(RegOpCode::Jump, 0, 0));
                    jump_patches.push((patch_pos, *target, false, 0));
                }
                CoreInst::JumpIfFalse(target, _) => {
                    if top > temp_base {
                        top -= 1;
                    }
                    let patch_pos = self.instructions.len();
                    self.emit(RegInstruction::encode_asbx(
                        RegOpCode::JumpIfFalse,
                        top as u8,
                        0,
                    ));
                    jump_patches.push((patch_pos, *target, true, top as u8));
                }
                CoreInst::Return { has_value, .. } => {
                    if *has_value && top > temp_base {
                        top -= 1;
                        self.emit(RegInstruction::encode_abc(
                            RegOpCode::Return,
                            top as u8,
                            1,
                            0,
                        ));
                    } else {
                        self.track_reg(top);
                        self.emit(RegInstruction::encode_abc(
                            RegOpCode::LoadNil,
                            top as u8,
                            0,
                            0,
                        ));
                        self.emit(RegInstruction::encode_abc(
                            RegOpCode::Return,
                            top as u8,
                            1,
                            0,
                        ));
                    }
                }
                CoreInst::Pop(_) => {
                    if top > temp_base {
                        top -= 1;
                    }
                }
                CoreInst::Dup(_) if top > temp_base => {
                    self.track_reg(top);
                    let prev = top - 1;
                    self.emit(RegInstruction::encode_abc(
                        RegOpCode::Move,
                        top as u8,
                        prev as u16,
                        0,
                    ));
                    top += 1;
                }
                _ => {}
            }
        }

        // Pass 2: Patch jump offsets
        let total_instructions = self.instructions.len();
        for (patch_idx, core_target, is_cond, reg_a) in jump_patches {
            let target_inst_idx = if core_target >= 0 && (core_target as usize) < core_to_reg.len()
            {
                core_to_reg[core_target as usize]
            } else {
                total_instructions
            };
            let offset = (target_inst_idx as i32) - (patch_idx as i32) - 1;
            let op = if is_cond {
                RegOpCode::JumpIfFalse
            } else {
                RegOpCode::Jump
            };
            self.instructions[patch_idx] = RegInstruction::encode_asbx(op, reg_a, offset);
        }

        // Ensure implicit return at end if not present
        if self
            .instructions
            .last()
            .copied()
            .and_then(RegInstruction::opcode)
            != Some(RegOpCode::Return)
        {
            self.emit(RegInstruction::encode_abc(RegOpCode::LoadNil, 0, 0, 0));
            self.emit(RegInstruction::encode_abc(RegOpCode::Return, 0, 1, 0));
        }

        RegCompiledFunction {
            name: func.name.clone(),
            instructions: self.instructions,
            constants: self.constants,
            num_registers: (self.max_reg + 1).max(1),
        }
    }

    fn emit(&mut self, inst: RegInstruction) {
        self.instructions.push(inst);
    }

    fn track_reg(&mut self, reg: usize) {
        if reg > self.max_reg {
            self.max_reg = reg;
        }
    }

    fn add_const(&mut self, c: Constant) -> usize {
        if let Some(pos) = self.constants.iter().position(|x| x == &c) {
            pos
        } else {
            let pos = self.constants.len();
            self.constants.push(c);
            pos
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_ir::{BinaryOp, CoreFunction, CoreInst};
    use aipo_source::SourceSpan;

    #[test]
    fn test_reg_emitter_basic_arithmetic() {
        let dummy_span = SourceSpan::default();
        let func = CoreFunction {
            name: "test".to_string(),
            is_async: false,
            params: vec!["a".to_string(), "b".to_string()],
            locals: vec!["c".to_string()],
            upvalues: Vec::new(),
            instructions: vec![
                CoreInst::Load("a".to_string(), dummy_span),
                CoreInst::Load("b".to_string(), dummy_span),
                CoreInst::Binary(BinaryOp::Add, dummy_span),
                CoreInst::Store("c".to_string(), dummy_span),
                CoreInst::Load("c".to_string(), dummy_span),
                CoreInst::Return {
                    has_value: true,
                    span: dummy_span,
                },
            ],
            span: dummy_span,
        };

        let compiled = RegEmitter::new().compile_function(&func);
        assert_eq!(compiled.name, "test");
        assert!(!compiled.instructions.is_empty());
        assert_eq!(
            compiled.instructions.last().unwrap().opcode(),
            Some(RegOpCode::Return)
        );
    }
}
