//! Disassembler for Aipo bytecode modules.

use crate::module::BytecodeModule;
use crate::opcode::{Constant, OpCode};
use aipo_source::Source;
use byteorder::{BigEndian, ByteOrder};

/// Disassembles a compiled bytecode module into a human-readable text listing.
#[must_use]
pub fn disassemble(module: &BytecodeModule) -> String {
    disassemble_impl(module, None)
}

/// Disassembles a compiled bytecode module with source line annotations.
#[must_use]
pub fn disassemble_with_source(module: &BytecodeModule, source: &Source) -> String {
    disassemble_impl(module, Some(source))
}

fn disassemble_impl(module: &BytecodeModule, source: Option<&Source>) -> String {
    let mut out = String::new();
    out.push_str("== Constants ==\n");
    for (i, c) in module.constants.iter().enumerate() {
        match c {
            Constant::Nil => out.push_str(&format!("  {:04} Nil\n", i)),
            Constant::Bool(b) => out.push_str(&format!("  {:04} Bool({b})\n", i)),
            Constant::Int(n) => out.push_str(&format!("  {:04} Int({n})\n", i)),
            Constant::Float(f) => out.push_str(&format!("  {:04} Float({f})\n", i)),
            Constant::String(s) => out.push_str(&format!("  {:04} String({s:?})\n", i)),
        }
    }

    out.push_str("\n== Names ==\n");
    for (i, name) in module.names.iter().enumerate() {
        out.push_str(&format!("  {:04} {name}\n", i));
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
                emit_inst(
                    &mut out,
                    offset,
                    &format!("UNKNOWN(0x{b:02x})"),
                    source,
                    module,
                );
                continue;
            }
        };

        let inst_str = match op {
            OpCode::Constant => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let val_str = module
                    .constants
                    .get(idx)
                    .map(|c| format!("{c:?}"))
                    .unwrap_or_else(|| "<invalid>".to_string());
                format!("OpConstant {idx:04} ({val_str})")
            }
            OpCode::GetGlobal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                format!("OpGetGlobal {idx:04} ({name})")
            }
            OpCode::SetGlobal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                format!("OpSetGlobal {idx:04} ({name})")
            }
            OpCode::GetField => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                format!("OpGetField {idx:04} ({name})")
            }
            OpCode::SetField => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                format!("OpSetField {idx:04} ({name})")
            }
            OpCode::Jump => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                format!("OpJump {rel:+} -> {target:04}")
            }
            OpCode::JumpIfFalse => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                format!("OpJumpIfFalse {rel:+} -> {target:04}")
            }
            OpCode::PushHandler => {
                let rel = BigEndian::read_i16(&module.code[cursor..cursor + 2]) as isize;
                let target = (offset as isize) + 3 + rel;
                cursor += 2;
                format!("OpPushHandler {rel:+} -> {target:04}")
            }
            OpCode::Call => {
                let count = module.code[cursor];
                cursor += 1;
                format!("OpCall {count}")
            }
            OpCode::BuildList => {
                let count = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                format!("OpBuildList {count}")
            }
            OpCode::BuildDict => {
                let count = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                format!("OpBuildDict {count}")
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
                format!("OpBuildStruct {name} (fields: {count}{suffix})")
            }
            OpCode::SealStruct => "OpSealStruct".to_string(),
            OpCode::AssertInvariant => {
                let type_idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(type_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("<invalid>");
                format!("OpAssertInvariant {name}")
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
                format!("OpAssertContract {position}: {name}{optional}{required}")
            }
            OpCode::CheckMutations => "OpCheckMutations".to_string(),
            OpCode::Await => "OpAwait".to_string(),
            OpCode::FillSelfCapture => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .names
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| "<invalid>".to_string());
                format!("OpFillSelfCapture ({name})")
            }
            OpCode::MakeFunction => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]) as usize;
                cursor += 2;
                let name = module
                    .functions
                    .get(idx)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| "<invalid>".to_string());
                format!("OpMakeFunction {idx:04} ({name})")
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
                format!("OpMakeClosure {idx:04} ({name}) upvalues={count}")
            }
            OpCode::GetUpvalue => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                format!("OpGetUpvalue {idx}")
            }
            OpCode::SetUpvalue => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                format!("OpSetUpvalue {idx}")
            }
            OpCode::GetLocal | OpCode::SetLocal => {
                let idx = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                cursor += 2;
                format!("{op:?} {idx}")
            }
            OpCode::JumpIfSetLocal => {
                let slot = BigEndian::read_u16(&module.code[cursor..cursor + 2]);
                let rel = BigEndian::read_i16(&module.code[cursor + 2..cursor + 4]);
                cursor += 4;
                format!("OpJumpIfSetLocal slot={slot} -> {rel:+}")
            }
            _ => format!("{op:?}"),
        };

        emit_inst(&mut out, offset, &inst_str, source, module);
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

fn emit_inst(
    out: &mut String,
    offset: usize,
    inst_str: &str,
    source: Option<&Source>,
    module: &BytecodeModule,
) {
    if let Some(src) = source {
        let loc_str = if let Some((_, span)) = module.spans.iter().find(|(o, _)| *o == offset) {
            match src.location(span.start) {
                Some(loc) => format!("[{}:{}]", loc.line, loc.column),
                None => format!("[{}..{}]", span.start, span.end),
            }
        } else {
            String::new()
        };
        out.push_str(&format!("{offset:04}  {loc_str:<10} {inst_str}\n"));
    } else {
        out.push_str(&format!("{offset:04}  {inst_str}\n"));
    }
}
