//! Disassembler for Aipo bytecode modules.

use crate::module::BytecodeModule;
use crate::opcode::{Constant, OpCode};
use byteorder::{BigEndian, ByteOrder};

/// Disassembles a compiled bytecode module into a human-readable text listing.
#[must_use]
pub fn disassemble(module: &BytecodeModule) -> String {
    let mut out = String::new();
    out.push_str("== Constants ==\n");
    for (i, c) in module.constants.iter().enumerate() {
        match c {
            Constant::Nil => out.push_str(&format!("  {:04} Nil\n", i)),
            Constant::Bool(b) => out.push_str(&format!("  {:04} Bool({})\n", i, b)),
            Constant::Int(n) => out.push_str(&format!("  {:04} Int({})\n", i, n)),
            Constant::Float(f) => out.push_str(&format!("  {:04} Float({})\n", i, f)),
            Constant::String(s) => out.push_str(&format!("  {:04} String({:?})\n", i, s)),
        }
    }

    out.push_str("\n== Names ==\n");
    for (i, name) in module.names.iter().enumerate() {
        out.push_str(&format!("  {:04} {}\n", i, name));
    }

    out.push_str("\n== Disassembly ==\n");
    let mut cursor = 0;
    while cursor < module.code.len() {
        let offset = cursor;
        let byte = module.code[cursor];
        cursor += 1;

        let op = match OpCode::try_from(byte) {
            Ok(op) => op,
            Err(b) => {
                out.push_str(&format!("{:04}  UNKNOWN(0x{:02x})\n", offset, b));
                continue;
            }
        };

        match op {
            OpCode::Constant => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let val_str = module
                    .constants
                    .get(idx)
                    .map(|c| format!("{:?}", c))
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!(
                    "{:04}  OpConstant {:04} ({})\n",
                    offset, idx, val_str
                ));
            }
            OpCode::GetGlobal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                out.push_str(&format!(
                    "{:04}  OpGetGlobal {:04} ({})\n",
                    offset, idx, name
                ));
            }
            OpCode::SetGlobal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                out.push_str(&format!(
                    "{:04}  OpSetGlobal {:04} ({})\n",
                    offset, idx, name
                ));
            }
            OpCode::GetField => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                out.push_str(&format!(
                    "{:04}  OpGetField {:04} ({})\n",
                    offset, idx, name
                ));
            }
            OpCode::SetField => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                out.push_str(&format!(
                    "{:04}  OpSetField {:04} ({})\n",
                    offset, idx, name
                ));
            }
            OpCode::Jump => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                out.push_str(&format!(
                    "{:04}  OpJump {:+} -> {:04}\n",
                    offset, rel, target
                ));
            }
            OpCode::JumpIfFalse => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                out.push_str(&format!(
                    "{:04}  OpJumpIfFalse {:+} -> {:04}\n",
                    offset, rel, target
                ));
            }
            OpCode::PushHandler => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                out.push_str(&format!(
                    "{:04}  OpPushHandler {:+} -> {:04}\n",
                    offset, rel, target
                ));
            }
            OpCode::Call => {
                let count = module.code[cursor];
                cursor += 1;
                out.push_str(&format!("{:04}  OpCall {}\n", offset, count));
            }
            OpCode::BuildList => {
                let count = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                out.push_str(&format!("{:04}  OpBuildList {}\n", offset, count));
            }
            OpCode::BuildDict => {
                let count = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                out.push_str(&format!("{:04}  OpBuildDict {}\n", offset, count));
            }
            OpCode::BuildStruct => {
                let type_idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                let count = BigEndian::read_u16(&module.code[cursor + 2..cursor + 4]);
                let defer_fixed = module.code[cursor + 4];
                cursor += 5;
                let name = module
                    .names
                    .get(type_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                let suffix = if defer_fixed == 0 {
                    ""
                } else {
                    ", constructing"
                };
                out.push_str(&format!(
                    "{:04}  OpBuildStruct {} (fields: {}{suffix})\n",
                    offset, name, count
                ));
            }
            OpCode::SealStruct => {
                out.push_str(&format!("{offset:04}  OpSealStruct\n"));
            }
            OpCode::AssertInvariant => {
                let type_idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(type_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                out.push_str(&format!("{offset:04}  OpAssertInvariant {name}\n"));
            }
            OpCode::AssertContract => {
                let type_idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                let nullable = module.code[cursor + 2];
                let position_idx =
                    BigEndian::read_u16(&module.code[cursor + 3..cursor + 5]) as usize;
                let operation_count = module.code[cursor + 5] as usize;
                let mut operations = Vec::with_capacity(operation_count);
                for index in 0..operation_count {
                    let base = cursor + 6 + index * 3;
                    let name_idx = BigEndian::read_u16(&module.code[base..base + 2]) as usize;
                    let arity = module.code[base + 2];
                    let name = module
                        .names
                        .get(name_idx)
                        .map(|s| s.as_str())
                        .unwrap_or("<invalid>");
                    operations.push(format!("{name}/{arity}"));
                }
                // Five fixed operand bytes plus the operation count, then three bytes per
                // operation (name index + arity).
                cursor += 6 + 3 * operation_count;
                let name = module
                    .names
                    .get(type_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                let position = module
                    .names
                    .get(position_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                let optional = if nullable == 0 { "" } else { "?" };
                let required = if operations.is_empty() {
                    String::new()
                } else {
                    format!(" {{{}}}", operations.join(", "))
                };
                out.push_str(&format!(
                    "{offset:04}  OpAssertContract {position}: {name}{optional}{required}\n"
                ));
            }
            OpCode::CheckMutations => {
                out.push_str(&format!("{offset:04}  OpCheckMutations\n"));
            }
            OpCode::FillSelfCapture => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!("{offset:04}  OpFillSelfCapture ({name})\n"));
            }
            OpCode::MakeFunction => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .functions
                    .get(idx)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!(
                    "{:04}  OpMakeFunction {:04} ({})\n",
                    offset, idx, name
                ));
            }
            OpCode::MakeClosure => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                let count = BigEndian::read_u16(&module.code[cursor + 2..cursor + 4]);
                cursor += 4;
                let name = module
                    .functions
                    .get(idx)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| "<invalid>".to_string());
                out.push_str(&format!(
                    "{:04}  OpMakeClosure {:04} ({}) upvalues={}\n",
                    offset, idx, name, count
                ));
            }
            OpCode::GetUpvalue => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                out.push_str(&format!("{:04}  OpGetUpvalue {}\n", offset, idx));
            }
            OpCode::SetUpvalue => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                out.push_str(&format!("{:04}  OpSetUpvalue {}\n", offset, idx));
            }
            OpCode::GetLocal | OpCode::SetLocal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                out.push_str(&format!("{:04}  {:?} {}\n", offset, op, idx));
            }
            OpCode::JumpIfSetLocal => {
                let slot = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                let rel = BigEndian::read_i16(&module.code[cursor + 2..cursor + 4]);
                cursor += 4;
                out.push_str(&format!(
                    "{:04}  OpJumpIfSetLocal slot={} -> {:+}\n",
                    offset, slot, rel
                ));
            }
            _ => {
                out.push_str(&format!("{:04}  {:?}\n", offset, op));
            }
        }
    }

    if !module.functions.is_empty() {
        out.push_str("\n== Functions ==\n");
        for (i, f) in module.functions.iter().enumerate() {
            out.push_str(&format!(
                "  {:04} {} @{:04} params={} locals={}\n",
                i, f.name, f.entry_ip, f.params, f.locals
            ));
        }
    }

    out
}
