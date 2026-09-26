//! WebAssembly compiler lowering Aipo HIR to standard Wasm binary modules (ADP-013).

use crate::emitter::WasmEmitter;
use crate::error::WasmCompileError;
use crate::types::{WasmFnType, WasmType};
use aipo_ast::{BinaryOp, Literal, UnaryOp};
use aipo_hir::{
    HirExpr, HirFunctionDecl, HirIfStmt, HirItem, HirParam, HirProgram, HirStmt, HirStructDecl,
};
use aipo_lexer::{parse_float_literal, parse_int_literal};
use aipo_source::SourceSpan;
use std::collections::HashMap;
use wasm_encoder::{BlockType, ConstExpr, Function, GlobalType, Instruction, MemArg, ValType};

/// Represents a frame in the WebAssembly structured control stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFrame {
    /// Outer block of a loop (target for forward `break`).
    LoopBreak,
    /// Loop header (target for backward `continue`).
    LoopContinue,
    /// Step/increment block of a `repeat` loop (target for `continue`).
    RepeatStep,
    /// Standard structured block (e.g., `if` condition or inline block).
    Block,
}

/// High-level type tag for local variables to disambiguate field accesses and string properties.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalKind {
    Int,
    Float,
    Bool,
    String,
    Struct(String),
}

/// Describes the byte layout of a struct in linear memory.
#[derive(Debug, Clone)]
struct StructLayout {
    size: u32,
    fields: HashMap<String, (u32, WasmType)>,
}

/// Compiles an `HirProgram` into a standard WebAssembly binary module (`.wasm`).
///
/// Functions declared in the program are compiled into Wasm functions and exported
/// under their declared names. Any top-level statements are compiled and exported
/// under the entrypoint symbol `__top_level__`.
///
/// # Errors
///
/// Returns a [`WasmCompileError`] if an unsupported language construct, unknown variable,
/// or type mismatch is encountered.
pub fn compile_hir(program: &HirProgram) -> Result<Vec<u8>, WasmCompileError> {
    let mut emitter = WasmEmitter::new();
    let mut functions = HashMap::new();
    let mut structs = HashMap::new();

    // Pass 0: Collect Struct definitions and compute layouts
    for item in &program.items {
        if let HirItem::Struct(decl) = item {
            register_struct_layout(decl, &mut structs);
        }
    }
    refine_struct_layouts(program, &mut structs);

    // Pass 0.5: Linear Memory and Static String Pool
    emitter.enable_memory(1, None);
    emitter.export_memory("memory");

    let mut static_strings = HashMap::new();
    let mut data_segments = Vec::new();
    let mut next_static_offset = 1024i32;

    collect_program_strings(
        program,
        &mut static_strings,
        &mut data_segments,
        &mut next_static_offset,
    );

    for (offset, bytes) in data_segments {
        emitter.add_data_segment(0, offset, bytes);
    }

    // Global 0: Dynamic Heap Pointer (`__aipo_heap_ptr`)
    let heap_start = next_static_offset;
    emitter.add_global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(heap_start),
    );

    // Function 0: Built-in Allocator `__aipo_alloc(size: i32) -> i32`
    let alloc_fn_type = WasmFnType::new(vec![WasmType::I32], vec![WasmType::I32]);
    let alloc_type_idx = emitter.add_type(alloc_fn_type.clone());
    let alloc_fn = build_allocator_function();
    let alloc_func_idx = emitter.add_function(alloc_type_idx, alloc_fn);
    emitter.export_function("__aipo_alloc", alloc_func_idx);
    functions.insert(
        "__aipo_alloc".to_string(),
        (alloc_func_idx, alloc_type_idx, alloc_fn_type),
    );

    // Pass 1: Collect signatures of all declared functions
    let mut func_decls: Vec<&HirFunctionDecl> = Vec::new();
    for item in &program.items {
        match item {
            HirItem::Fn(func) => {
                if func.is_async {
                    return Err(WasmCompileError::UnsupportedItem {
                        message: "async functions are scheduled for Milestone 5 (ADP-013)".into(),
                        span: func.span,
                    });
                }
                let params = func.params.iter().map(param_wasm_type).collect::<Vec<_>>();
                let ret = resolve_return_type(
                    &func.return_type,
                    &func.params,
                    &func.body,
                    &functions,
                    &structs,
                );
                let fn_type = WasmFnType::new(params, ret.into_iter().collect());
                let type_idx = emitter.add_type(fn_type.clone());
                let func_idx = functions.len() as u32;
                functions.insert(func.name.clone(), (func_idx, type_idx, fn_type));
                func_decls.push(func);
            }
            HirItem::Struct(_) | HirItem::Export(_) | HirItem::Import(_) => {
                // Metadata items handled or deferred
            }
            other => {
                return Err(WasmCompileError::UnsupportedItem {
                    message: format!("item `{other:?}` is not yet supported in Wasm backend"),
                    span: program.span,
                });
            }
        }
    }

    // Top-level statements become an entrypoint function `__top_level__`
    let top_level_info = if !program.statements.is_empty() {
        let mut top_locals = HashMap::new();
        let ret =
            infer_body_return_type(&program.statements, &mut top_locals, &functions, &structs);
        let fn_type = WasmFnType::new(vec![], ret.into_iter().collect());
        let type_idx = emitter.add_type(fn_type.clone());
        let func_idx = functions.len() as u32;
        functions.insert("__top_level__".to_string(), (func_idx, type_idx, fn_type));
        Some((func_idx, type_idx, ret))
    } else {
        None
    };

    // Pass 2: Compile the body of each function
    for func in func_decls {
        let (_, type_idx, fn_type) = &functions[&func.name];
        let compiled_fn = compile_function_body(
            &func.name,
            &func.params,
            fn_type.results.first().copied(),
            &func.body,
            &functions,
            &structs,
            &static_strings,
            alloc_func_idx,
        )?;
        let assigned_idx = emitter.add_function(*type_idx, compiled_fn);
        emitter.export_function(&func.name, assigned_idx);
    }

    // Compile top-level entrypoint if present
    if let Some((_, type_idx, ret)) = top_level_info {
        let compiled_fn = compile_function_body(
            "__top_level__",
            &[],
            ret,
            &program.statements,
            &functions,
            &structs,
            &static_strings,
            alloc_func_idx,
        )?;
        let assigned_idx = emitter.add_function(type_idx, compiled_fn);
        emitter.export_function("__top_level__", assigned_idx);
    }

    Ok(emitter.finish())
}

/// Registers struct layout in linear memory (fields aligned to 8 bytes).
fn register_struct_layout(decl: &HirStructDecl, structs: &mut HashMap<String, StructLayout>) {
    let mut fields = HashMap::new();
    let mut offset = 0u32;
    for f in &decl.fields {
        let ty = if let Some(ref def) = f.default {
            match def {
                HirExpr::Literal(Literal::Float(..), _) => WasmType::F64,
                HirExpr::Literal(Literal::Bool(..), _) => WasmType::I32,
                _ => WasmType::I64,
            }
        } else {
            WasmType::I64
        };
        fields.insert(f.name.clone(), (offset, ty));
        offset += 8;
    }
    let size = if offset == 0 { 8 } else { offset };
    structs.insert(decl.name.clone(), StructLayout { size, fields });
}

/// Scans program expressions to refine field types in `structs` based on constructor literals.
fn refine_struct_layouts(program: &HirProgram, structs: &mut HashMap<String, StructLayout>) {
    for item in &program.items {
        if let HirItem::Fn(func) = item {
            scan_stmts_for_constructs(&func.body, structs);
        }
    }
    scan_stmts_for_constructs(&program.statements, structs);
}

fn scan_stmts_for_constructs(stmts: &[HirStmt], structs: &mut HashMap<String, StructLayout>) {
    for stmt in stmts {
        match stmt {
            HirStmt::Let(_, expr, _) | HirStmt::Var(_, expr, _) | HirStmt::Expr(expr) => {
                scan_expr_for_constructs(expr, structs);
            }
            HirStmt::Assign(target, expr, _) | HirStmt::CompoundAssign(_, target, expr, _) => {
                scan_expr_for_constructs(target, structs);
                scan_expr_for_constructs(expr, structs);
            }
            HirStmt::Return(Some(expr), _) => {
                scan_expr_for_constructs(expr, structs);
            }
            HirStmt::If(s) => {
                scan_expr_for_constructs(&s.condition, structs);
                scan_stmts_for_constructs(&s.then_branch, structs);
                for (c, b) in &s.elif_branches {
                    scan_expr_for_constructs(c, structs);
                    scan_stmts_for_constructs(b, structs);
                }
                if let Some(eb) = &s.else_branch {
                    scan_stmts_for_constructs(eb, structs);
                }
            }
            HirStmt::While(cond, body, _) => {
                scan_expr_for_constructs(cond, structs);
                scan_stmts_for_constructs(body, structs);
            }
            HirStmt::Loop(body, _) => {
                scan_stmts_for_constructs(body, structs);
            }
            HirStmt::Repeat(count, _, body, _) => {
                scan_expr_for_constructs(count, structs);
                scan_stmts_for_constructs(body, structs);
            }
            _ => {}
        }
    }
}

fn scan_expr_for_constructs(expr: &HirExpr, structs: &mut HashMap<String, StructLayout>) {
    match expr {
        HirExpr::Construct(type_name, fields, _) => {
            let mut float_fields = Vec::new();
            for (maybe_name, field_expr) in fields {
                scan_expr_for_constructs(field_expr, structs);
                if let Some(name) = maybe_name {
                    if matches!(field_expr, HirExpr::Literal(Literal::Float(..), _)) {
                        float_fields.push(name.clone());
                    }
                }
            }
            if let Some(layout) = structs.get_mut(type_name) {
                for name in float_fields {
                    if let Some((_, field_ty)) = layout.fields.get_mut(&name) {
                        *field_ty = WasmType::F64;
                    }
                }
            }
        }
        HirExpr::Binary(_, left, right, _) => {
            scan_expr_for_constructs(left, structs);
            scan_expr_for_constructs(right, structs);
        }
        HirExpr::Unary(_, inner, _) => {
            scan_expr_for_constructs(inner, structs);
        }
        HirExpr::If(cond, then_e, else_e, _) => {
            scan_expr_for_constructs(cond, structs);
            scan_expr_for_constructs(then_e, structs);
            scan_expr_for_constructs(else_e, structs);
        }
        HirExpr::Call(callee, args, _) => {
            scan_expr_for_constructs(callee, structs);
            for arg in args {
                scan_expr_for_constructs(&arg.value, structs);
            }
        }
        HirExpr::Dot(receiver, _, _) => {
            scan_expr_for_constructs(receiver, structs);
        }
        _ => {}
    }
}

/// Builds the linear memory bump allocator function `__aipo_alloc(size: i32) -> i32`.
fn build_allocator_function() -> Function {
    let mut alloc_fn = Function::new([(2, ValType::I32)]);
    // local 0: size (param)
    // local 1: old_ptr
    // local 2: new_ptr

    // Align size to multiple of 8: size = (size + 7) & -8
    alloc_fn.instruction(&Instruction::LocalGet(0));
    alloc_fn.instruction(&Instruction::I32Const(7));
    alloc_fn.instruction(&Instruction::I32Add);
    alloc_fn.instruction(&Instruction::I32Const(-8));
    alloc_fn.instruction(&Instruction::I32And);
    alloc_fn.instruction(&Instruction::LocalSet(0));

    // old_ptr = global.get 0
    alloc_fn.instruction(&Instruction::GlobalGet(0));
    alloc_fn.instruction(&Instruction::LocalSet(1));

    // new_ptr = old_ptr + size
    alloc_fn.instruction(&Instruction::LocalGet(1));
    alloc_fn.instruction(&Instruction::LocalGet(0));
    alloc_fn.instruction(&Instruction::I32Add);
    alloc_fn.instruction(&Instruction::LocalSet(2));

    // Check if new_ptr > (memory.size << 16)
    // If so, grow memory dynamically
    alloc_fn.instruction(&Instruction::LocalGet(2));
    alloc_fn.instruction(&Instruction::MemorySize(0));
    alloc_fn.instruction(&Instruction::I32Const(16));
    alloc_fn.instruction(&Instruction::I32Shl);
    alloc_fn.instruction(&Instruction::I32GtU);
    alloc_fn.instruction(&Instruction::If(BlockType::Empty));

    // delta_pages = ((new_ptr - (memory.size << 16)) + 65535) >> 16
    alloc_fn.instruction(&Instruction::LocalGet(2));
    alloc_fn.instruction(&Instruction::MemorySize(0));
    alloc_fn.instruction(&Instruction::I32Const(16));
    alloc_fn.instruction(&Instruction::I32Shl);
    alloc_fn.instruction(&Instruction::I32Sub);
    alloc_fn.instruction(&Instruction::I32Const(65535));
    alloc_fn.instruction(&Instruction::I32Add);
    alloc_fn.instruction(&Instruction::I32Const(16));
    alloc_fn.instruction(&Instruction::I32ShrU);
    alloc_fn.instruction(&Instruction::MemoryGrow(0));
    alloc_fn.instruction(&Instruction::I32Const(-1));
    alloc_fn.instruction(&Instruction::I32Eq);
    alloc_fn.instruction(&Instruction::If(BlockType::Empty));
    alloc_fn.instruction(&Instruction::Unreachable); // Trap on out-of-memory
    alloc_fn.instruction(&Instruction::End);
    alloc_fn.instruction(&Instruction::End);

    // global.set 0 new_ptr
    alloc_fn.instruction(&Instruction::LocalGet(2));
    alloc_fn.instruction(&Instruction::GlobalSet(0));

    // return old_ptr
    alloc_fn.instruction(&Instruction::LocalGet(1));
    alloc_fn.instruction(&Instruction::End);

    alloc_fn
}

/// Collects static string literals into the linear memory data segment pool.
fn collect_program_strings(
    program: &HirProgram,
    static_strings: &mut HashMap<String, i32>,
    data_segments: &mut Vec<(i32, Vec<u8>)>,
    next_offset: &mut i32,
) {
    for item in &program.items {
        if let HirItem::Fn(func) = item {
            collect_stmts_strings(&func.body, static_strings, data_segments, next_offset);
        }
    }
    collect_stmts_strings(
        &program.statements,
        static_strings,
        data_segments,
        next_offset,
    );
}

fn collect_stmts_strings(
    stmts: &[HirStmt],
    static_strings: &mut HashMap<String, i32>,
    data_segments: &mut Vec<(i32, Vec<u8>)>,
    next_offset: &mut i32,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Let(_, expr, _) | HirStmt::Var(_, expr, _) | HirStmt::Expr(expr) => {
                collect_expr_strings(expr, static_strings, data_segments, next_offset);
            }
            HirStmt::Assign(target, expr, _) | HirStmt::CompoundAssign(_, target, expr, _) => {
                collect_expr_strings(target, static_strings, data_segments, next_offset);
                collect_expr_strings(expr, static_strings, data_segments, next_offset);
            }
            HirStmt::Return(Some(expr), _) => {
                collect_expr_strings(expr, static_strings, data_segments, next_offset);
            }
            HirStmt::If(s) => {
                collect_expr_strings(&s.condition, static_strings, data_segments, next_offset);
                collect_stmts_strings(&s.then_branch, static_strings, data_segments, next_offset);
                for (cond, body) in &s.elif_branches {
                    collect_expr_strings(cond, static_strings, data_segments, next_offset);
                    collect_stmts_strings(body, static_strings, data_segments, next_offset);
                }
                if let Some(else_branch) = &s.else_branch {
                    collect_stmts_strings(else_branch, static_strings, data_segments, next_offset);
                }
            }
            HirStmt::While(cond, body, _) => {
                collect_expr_strings(cond, static_strings, data_segments, next_offset);
                collect_stmts_strings(body, static_strings, data_segments, next_offset);
            }
            HirStmt::Loop(body, _) => {
                collect_stmts_strings(body, static_strings, data_segments, next_offset);
            }
            HirStmt::Repeat(count, _, body, _) => {
                collect_expr_strings(count, static_strings, data_segments, next_offset);
                collect_stmts_strings(body, static_strings, data_segments, next_offset);
            }
            _ => {}
        }
    }
}

fn collect_expr_strings(
    expr: &HirExpr,
    static_strings: &mut HashMap<String, i32>,
    data_segments: &mut Vec<(i32, Vec<u8>)>,
    next_offset: &mut i32,
) {
    match expr {
        HirExpr::Literal(Literal::String(text, _), _) => {
            if !static_strings.contains_key(text) {
                let offset = *next_offset;
                let len = text.len() as i32;
                let mut bytes = Vec::with_capacity(4 + text.len());
                bytes.extend_from_slice(&len.to_le_bytes());
                bytes.extend_from_slice(text.as_bytes());

                data_segments.push((offset, bytes));
                static_strings.insert(text.clone(), offset);

                *next_offset += (4 + text.len() as i32 + 7) & !7;
            }
        }
        HirExpr::Binary(_, left, right, _) => {
            collect_expr_strings(left, static_strings, data_segments, next_offset);
            collect_expr_strings(right, static_strings, data_segments, next_offset);
        }
        HirExpr::Unary(_, inner, _) => {
            collect_expr_strings(inner, static_strings, data_segments, next_offset);
        }
        HirExpr::If(cond, then_e, else_e, _) => {
            collect_expr_strings(cond, static_strings, data_segments, next_offset);
            collect_expr_strings(then_e, static_strings, data_segments, next_offset);
            collect_expr_strings(else_e, static_strings, data_segments, next_offset);
        }
        HirExpr::Call(callee, args, _) => {
            collect_expr_strings(callee, static_strings, data_segments, next_offset);
            for arg in args {
                collect_expr_strings(&arg.value, static_strings, data_segments, next_offset);
            }
        }
        HirExpr::Construct(_, fields, _) => {
            for (_, field_expr) in fields {
                collect_expr_strings(field_expr, static_strings, data_segments, next_offset);
            }
        }
        HirExpr::Dot(receiver, _, _) => {
            collect_expr_strings(receiver, static_strings, data_segments, next_offset);
        }
        _ => {}
    }
}

/// Resolves the Wasm type for a function parameter.
fn param_wasm_type(param: &HirParam) -> WasmType {
    if let Some(ty) = &param.type_annotation {
        match ty.name.as_str() {
            "Float" => WasmType::F64,
            "Bool" => WasmType::I32,
            _ => WasmType::I64,
        }
    } else {
        WasmType::I64
    }
}

/// Resolves the return type from explicit annotation or infers it from the function body.
fn resolve_return_type(
    annot: &Option<aipo_ast::TypeAnnotation>,
    params: &[HirParam],
    body: &[HirStmt],
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    structs: &HashMap<String, StructLayout>,
) -> Option<WasmType> {
    if let Some(ty) = annot {
        match ty.name.as_str() {
            "Float" => Some(WasmType::F64),
            "Bool" => Some(WasmType::I32),
            "None" => None,
            "String" => Some(WasmType::I32),
            s if structs.contains_key(s) => Some(WasmType::I32),
            _ => Some(WasmType::I64),
        }
    } else {
        let mut locals = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            let ty = param_wasm_type(param);
            let kind = if let Some(t) = &param.type_annotation {
                match t.name.as_str() {
                    "Float" => LocalKind::Float,
                    "Bool" => LocalKind::Bool,
                    "String" => LocalKind::String,
                    other if structs.contains_key(other) => LocalKind::Struct(other.to_string()),
                    _ => LocalKind::Int,
                }
            } else {
                LocalKind::Int
            };
            locals.insert(param.name.clone(), (i as u32, ty, kind));
        }
        infer_body_return_type(body, &mut locals, functions, structs)
    }
}

/// Scans statements to deduce the return type of a block or function body.
fn infer_body_return_type(
    body: &[HirStmt],
    locals: &mut HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    structs: &HashMap<String, StructLayout>,
) -> Option<WasmType> {
    let mut has_return = false;
    for stmt in body {
        match stmt {
            HirStmt::Let(name, expr, _) | HirStmt::Var(name, expr, _) => {
                let ty = infer_expr_type(expr, locals, functions, structs).unwrap_or(WasmType::I64);
                let kind = infer_expr_kind(expr, locals);
                let idx = locals.len() as u32;
                locals.insert(name.clone(), (idx, ty, kind));
            }
            HirStmt::Return(Some(expr), _) => {
                has_return = true;
                if let Ok(ty) = infer_expr_type(expr, locals, functions, structs) {
                    return Some(ty);
                }
            }
            HirStmt::Return(None, _) => {
                return None;
            }
            HirStmt::If(s) => {
                if let Some(ty) = infer_body_return_type(&s.then_branch, locals, functions, structs)
                {
                    return Some(ty);
                }
                for (_, elif_body) in &s.elif_branches {
                    if let Some(ty) = infer_body_return_type(elif_body, locals, functions, structs)
                    {
                        return Some(ty);
                    }
                }
                if let Some(else_branch) = &s.else_branch {
                    if let Some(ty) =
                        infer_body_return_type(else_branch, locals, functions, structs)
                    {
                        return Some(ty);
                    }
                }
            }
            HirStmt::While(_, loop_body, _)
            | HirStmt::Loop(loop_body, _)
            | HirStmt::Repeat(_, _, loop_body, _) => {
                if let Some(ty) = infer_body_return_type(loop_body, locals, functions, structs) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }
    // If the last statement is an expression statement, its type is the implicit return
    if let Some(HirStmt::Expr(expr)) = body.last() {
        if let Ok(ty) = infer_expr_type(expr, locals, functions, structs) {
            return Some(ty);
        }
    }
    if has_return {
        Some(WasmType::I64)
    } else {
        None
    }
}

/// Recursively scans statements to register all declared locals and helpers.
fn pre_scan_stmts(
    stmts: &[HirStmt],
    params_count: usize,
    locals: &mut HashMap<String, (u32, WasmType, LocalKind)>,
    declared_locals: &mut Vec<WasmType>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    structs: &HashMap<String, StructLayout>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    for stmt in stmts {
        match stmt {
            HirStmt::Let(name, init_expr, _) | HirStmt::Var(name, init_expr, _) => {
                let ty =
                    infer_expr_type(init_expr, locals, functions, structs).unwrap_or(WasmType::I64);
                let kind = infer_expr_kind(init_expr, locals);
                let idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(ty);
                locals.insert(name.clone(), (idx, ty, kind));
            }
            HirStmt::If(s) => {
                pre_scan_stmts(
                    &s.then_branch,
                    params_count,
                    locals,
                    declared_locals,
                    functions,
                    structs,
                    repeat_id,
                )?;
                for (_, elif_body) in &s.elif_branches {
                    pre_scan_stmts(
                        elif_body,
                        params_count,
                        locals,
                        declared_locals,
                        functions,
                        structs,
                        repeat_id,
                    )?;
                }
                if let Some(else_branch) = &s.else_branch {
                    pre_scan_stmts(
                        else_branch,
                        params_count,
                        locals,
                        declared_locals,
                        functions,
                        structs,
                        repeat_id,
                    )?;
                }
            }
            HirStmt::While(_, body, _) | HirStmt::Loop(body, _) => {
                pre_scan_stmts(
                    body,
                    params_count,
                    locals,
                    declared_locals,
                    functions,
                    structs,
                    repeat_id,
                )?;
            }
            HirStmt::Repeat(_, maybe_index, body, _) => {
                *repeat_id += 1;
                let id = *repeat_id;
                // repeat_limit slot
                let limit_idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(WasmType::I64);
                locals.insert(
                    format!("__repeat_limit_{id}"),
                    (limit_idx, WasmType::I64, LocalKind::Int),
                );
                // repeat_idx slot
                let idx_idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(WasmType::I64);
                locals.insert(
                    format!("__repeat_idx_{id}"),
                    (idx_idx, WasmType::I64, LocalKind::Int),
                );
                // user index variable if specified
                if let Some(name) = maybe_index {
                    let user_idx = (params_count + declared_locals.len()) as u32;
                    declared_locals.push(WasmType::I64);
                    locals.insert(name.clone(), (user_idx, WasmType::I64, LocalKind::Int));
                }
                pre_scan_stmts(
                    body,
                    params_count,
                    locals,
                    declared_locals,
                    functions,
                    structs,
                    repeat_id,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Infers the high-level kind of an expression for local tracking.
fn infer_expr_kind(
    expr: &HirExpr,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
) -> LocalKind {
    match expr {
        HirExpr::Literal(Literal::Int(..), _) => LocalKind::Int,
        HirExpr::Literal(Literal::Float(..), _) => LocalKind::Float,
        HirExpr::Literal(Literal::Bool(..), _) => LocalKind::Bool,
        HirExpr::Literal(Literal::String(..), _) => LocalKind::String,
        HirExpr::Construct(name, ..) => LocalKind::Struct(name.clone()),
        HirExpr::Identifier(name, _) => locals
            .get(name)
            .map(|(_, _, k)| k.clone())
            .unwrap_or(LocalKind::Int),
        _ => LocalKind::Int,
    }
}

/// Compiles a single function's body into a WebAssembly `Function`.
#[allow(clippy::too_many_arguments)]
fn compile_function_body(
    _func_name: &str,
    params: &[HirParam],
    return_type: Option<WasmType>,
    body: &[HirStmt],
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
) -> Result<Function, WasmCompileError> {
    let mut locals: HashMap<String, (u32, WasmType, LocalKind)> = HashMap::new();
    let mut declared_locals: Vec<WasmType> = Vec::new();

    // Map parameters to locals 0..params.len()
    for (i, param) in params.iter().enumerate() {
        let ty = param_wasm_type(param);
        let kind = if let Some(t) = &param.type_annotation {
            match t.name.as_str() {
                "Float" => LocalKind::Float,
                "Bool" => LocalKind::Bool,
                "String" => LocalKind::String,
                other if structs.contains_key(other) => LocalKind::Struct(other.to_string()),
                _ => LocalKind::Int,
            }
        } else {
            LocalKind::Int
        };
        locals.insert(param.name.clone(), (i as u32, ty, kind));
    }

    // Allocate 8 helper scratch locals for struct construction nesting
    for d in 0..8 {
        let idx = (params.len() + declared_locals.len()) as u32;
        declared_locals.push(WasmType::I32);
        locals.insert(
            format!("__struct_temp_{d}"),
            (idx, WasmType::I32, LocalKind::Int),
        );
    }

    let dot_temp_idx = (params.len() + declared_locals.len()) as u32;
    declared_locals.push(WasmType::I32);
    locals.insert(
        "__dot_temp".to_string(),
        (dot_temp_idx, WasmType::I32, LocalKind::Int),
    );

    // Pre-scan pass to register all local variable declarations (let/var/repeat)
    let mut pre_scan_repeat_id = 0;
    pre_scan_stmts(
        body,
        params.len(),
        &mut locals,
        &mut declared_locals,
        functions,
        structs,
        &mut pre_scan_repeat_id,
    )?;

    // Build the Wasm function with its locals declaration
    let locals_spec = compress_locals(&declared_locals);
    let mut func = Function::new(locals_spec);
    let mut control_stack: Vec<ControlFrame> = Vec::new();
    let mut emit_repeat_id = 0;

    let terminated = compile_stmts(
        body,
        &mut func,
        &locals,
        functions,
        &mut control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        return_type,
        true,
        &mut emit_repeat_id,
        0,
    )?;

    // Ensure valid Wasm block termination
    if !terminated {
        if let Some(ret_ty) = return_type {
            match ret_ty {
                WasmType::I64 => {
                    func.instruction(&Instruction::I64Const(0));
                }
                WasmType::F64 => {
                    func.instruction(&Instruction::F64Const(0.0.into()));
                }
                WasmType::I32 => {
                    func.instruction(&Instruction::I32Const(0));
                }
            }
        }
    }

    func.instruction(&Instruction::End);
    Ok(func)
}

/// Compiles a slice of statements into a WebAssembly `Function`.
#[allow(clippy::too_many_arguments)]
fn compile_stmts(
    stmts: &[HirStmt],
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    return_type: Option<WasmType>,
    is_top_level: bool,
    repeat_id: &mut u32,
    struct_depth: usize,
) -> Result<bool, WasmCompileError> {
    let mut terminated = false;
    let stmt_count = stmts.len();

    for (idx, stmt) in stmts.iter().enumerate() {
        let is_last = idx + 1 == stmt_count;
        match stmt {
            HirStmt::Let(name, init_expr, _) | HirStmt::Var(name, init_expr, _) => {
                let (local_idx, local_ty, _) = locals[name];
                let expr_ty = compile_expr(
                    init_expr,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    struct_depth,
                )?;
                coerce_type(func, expr_ty, local_ty);
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            HirStmt::Assign(target, expr, span) => match target {
                HirExpr::Identifier(name, _) => {
                    if let Some(&(local_idx, target_ty, _)) = locals.get(name) {
                        let expr_ty = compile_expr(
                            expr,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        coerce_type(func, expr_ty, target_ty);
                        func.instruction(&Instruction::LocalSet(local_idx));
                    } else {
                        return Err(WasmCompileError::UnknownVariable {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                }
                HirExpr::Dot(receiver, field_name, _) => {
                    let receiver_struct_name = match &**receiver {
                        HirExpr::Identifier(name, _) => {
                            locals.get(name).and_then(|(_, _, kind)| match kind {
                                LocalKind::Struct(s_name) => Some(s_name.as_str()),
                                _ => None,
                            })
                        }
                        _ => None,
                    };
                    let field_info = if let Some(s_name) = receiver_struct_name {
                        structs
                            .get(s_name)
                            .and_then(|s| s.fields.get(field_name).copied())
                    } else {
                        structs
                            .values()
                            .find_map(|s| s.fields.get(field_name).copied())
                    };

                    if let Some((field_offset, field_ty)) = field_info {
                        let r_ty = compile_expr(
                            receiver,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        coerce_type(func, r_ty, WasmType::I32);
                        let val_ty = compile_expr(
                            expr,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        coerce_type(func, val_ty, field_ty);
                        match field_ty {
                            WasmType::F64 => {
                                func.instruction(&Instruction::F64Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I32 => {
                                func.instruction(&Instruction::I32Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 2,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I64 => {
                                func.instruction(&Instruction::I64Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                        }
                    } else {
                        return Err(WasmCompileError::UnsupportedExpr {
                            message: format!("unknown field `{field_name}` on struct"),
                            span: *span,
                        });
                    }
                }
                _ => {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "unsupported assignment target in Wasm backend".into(),
                        span: *span,
                    });
                }
            },
            HirStmt::CompoundAssign(op, target, expr, span) => match target {
                HirExpr::Identifier(name, _) => {
                    if let Some(&(local_idx, target_ty, _)) = locals.get(name) {
                        func.instruction(&Instruction::LocalGet(local_idx));
                        let expr_ty = compile_expr(
                            expr,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        emit_binary_op(func, *op, target_ty, expr_ty, *span)?;
                        func.instruction(&Instruction::LocalSet(local_idx));
                    } else {
                        return Err(WasmCompileError::UnknownVariable {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                }
                HirExpr::Dot(receiver, field_name, _) => {
                    let receiver_struct_name = match &**receiver {
                        HirExpr::Identifier(name, _) => {
                            locals.get(name).and_then(|(_, _, kind)| match kind {
                                LocalKind::Struct(s_name) => Some(s_name.as_str()),
                                _ => None,
                            })
                        }
                        _ => None,
                    };
                    let field_info = if let Some(s_name) = receiver_struct_name {
                        structs
                            .get(s_name)
                            .and_then(|s| s.fields.get(field_name).copied())
                    } else {
                        structs
                            .values()
                            .find_map(|s| s.fields.get(field_name).copied())
                    };

                    if let Some((field_offset, field_ty)) = field_info {
                        let dot_temp = locals["__dot_temp"].0;
                        let r_ty = compile_expr(
                            receiver,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        coerce_type(func, r_ty, WasmType::I32);
                        func.instruction(&Instruction::LocalSet(dot_temp));

                        func.instruction(&Instruction::LocalGet(dot_temp));
                        func.instruction(&Instruction::LocalGet(dot_temp));
                        match field_ty {
                            WasmType::F64 => {
                                func.instruction(&Instruction::F64Load(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I32 => {
                                func.instruction(&Instruction::I32Load(MemArg {
                                    offset: field_offset as u64,
                                    align: 2,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I64 => {
                                func.instruction(&Instruction::I64Load(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                        }

                        let expr_ty = compile_expr(
                            expr,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        emit_binary_op(func, *op, field_ty, expr_ty, *span)?;
                        match field_ty {
                            WasmType::F64 => {
                                func.instruction(&Instruction::F64Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I32 => {
                                func.instruction(&Instruction::I32Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 2,
                                    memory_index: 0,
                                }));
                            }
                            WasmType::I64 => {
                                func.instruction(&Instruction::I64Store(MemArg {
                                    offset: field_offset as u64,
                                    align: 3,
                                    memory_index: 0,
                                }));
                            }
                        }
                    } else {
                        return Err(WasmCompileError::UnsupportedExpr {
                            message: format!("unknown field `{field_name}` on struct"),
                            span: *span,
                        });
                    }
                }
                _ => {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "unsupported compound assignment target in Wasm backend".into(),
                        span: *span,
                    });
                }
            },
            HirStmt::Return(maybe_expr, span) => {
                if let Some(expr) = maybe_expr {
                    let expr_ty = compile_expr(
                        expr,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    if let Some(expected_ret) = return_type {
                        coerce_type(func, expr_ty, expected_ret);
                    }
                } else if return_type.is_some() {
                    return Err(WasmCompileError::TypeMismatch {
                        expected: format!("{return_type:?}"),
                        found: "void".into(),
                        span: *span,
                    });
                }
                func.instruction(&Instruction::Return);
                terminated = true;
            }
            HirStmt::If(if_stmt) => {
                compile_if_stmt(
                    if_stmt,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    return_type,
                    repeat_id,
                    struct_depth,
                )?;
            }
            HirStmt::While(cond, loop_body, _) => {
                compile_while_stmt(
                    cond,
                    loop_body,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    return_type,
                    repeat_id,
                    struct_depth,
                )?;
            }
            HirStmt::Loop(loop_body, _) => {
                compile_loop_stmt(
                    loop_body,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    return_type,
                    repeat_id,
                    struct_depth,
                )?;
            }
            HirStmt::Repeat(count, maybe_index, loop_body, _) => {
                compile_repeat_stmt(
                    count,
                    maybe_index.as_deref(),
                    loop_body,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    return_type,
                    repeat_id,
                    struct_depth,
                )?;
            }
            HirStmt::Break(span) => {
                if let Some(pos) = control_stack
                    .iter()
                    .rev()
                    .position(|f| *f == ControlFrame::LoopBreak)
                {
                    func.instruction(&Instruction::Br(pos as u32));
                } else {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "break outside of loop".into(),
                        span: *span,
                    });
                }
            }
            HirStmt::Continue(span) => {
                if let Some(pos) = control_stack.iter().rev().position(|f| {
                    *f == ControlFrame::RepeatStep || *f == ControlFrame::LoopContinue
                }) {
                    func.instruction(&Instruction::Br(pos as u32));
                } else {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "continue outside of loop".into(),
                        span: *span,
                    });
                }
            }
            HirStmt::Expr(expr) => {
                let expr_ty = compile_expr(
                    expr,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    struct_depth,
                )?;
                if is_top_level && is_last && return_type.is_some() {
                    if let Some(expected_ret) = return_type {
                        coerce_type(func, expr_ty, expected_ret);
                    }
                    terminated = true;
                } else {
                    func.instruction(&Instruction::Drop);
                }
            }
            other => {
                return Err(WasmCompileError::UnsupportedStmt {
                    message: format!("statement `{other:?}` is scheduled for Milestone 4"),
                    span: program_stmt_span(other),
                });
            }
        }
    }

    Ok(terminated)
}

/// Compiles an `if ... elif ... else` statement block.
#[allow(clippy::too_many_arguments)]
fn compile_if_stmt(
    if_stmt: &HirIfStmt,
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
    struct_depth: usize,
) -> Result<(), WasmCompileError> {
    let cond_ty = compile_expr(
        &if_stmt.condition,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        struct_depth,
    )?;
    coerce_to_bool(func, cond_ty);

    control_stack.push(ControlFrame::Block);
    func.instruction(&Instruction::If(BlockType::Empty));
    compile_stmts(
        &if_stmt.then_branch,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        return_type,
        false,
        repeat_id,
        struct_depth,
    )?;

    let has_elifs = !if_stmt.elif_branches.is_empty();
    let has_else = if_stmt.else_branch.is_some();

    if has_elifs || has_else {
        func.instruction(&Instruction::Else);

        let mut elif_count = 0;
        for (elif_cond, elif_body) in &if_stmt.elif_branches {
            let e_cond_ty = compile_expr(
                elif_cond,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                struct_depth,
            )?;
            coerce_to_bool(func, e_cond_ty);

            control_stack.push(ControlFrame::Block);
            func.instruction(&Instruction::If(BlockType::Empty));
            compile_stmts(
                elif_body,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                return_type,
                false,
                repeat_id,
                struct_depth,
            )?;
            func.instruction(&Instruction::Else);
            elif_count += 1;
        }

        if let Some(else_branch) = &if_stmt.else_branch {
            compile_stmts(
                else_branch,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                return_type,
                false,
                repeat_id,
                struct_depth,
            )?;
        }

        for _ in 0..elif_count {
            control_stack.pop();
            func.instruction(&Instruction::End);
        }
    }

    control_stack.pop();
    func.instruction(&Instruction::End);
    Ok(())
}

/// Compiles a `while condition { body }` loop.
#[allow(clippy::too_many_arguments)]
fn compile_while_stmt(
    cond: &HirExpr,
    loop_body: &[HirStmt],
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
    struct_depth: usize,
) -> Result<(), WasmCompileError> {
    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    let cond_ty = compile_expr(
        cond,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        struct_depth,
    )?;
    coerce_to_bool(func, cond_ty);
    func.instruction(&Instruction::I32Eqz);

    let break_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopBreak)
        .unwrap() as u32;
    func.instruction(&Instruction::BrIf(break_depth));

    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        return_type,
        false,
        repeat_id,
        struct_depth,
    )?;

    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    control_stack.pop();
    func.instruction(&Instruction::End);
    control_stack.pop();
    func.instruction(&Instruction::End);

    Ok(())
}

/// Compiles an unconditional `loop { body }` structure.
#[allow(clippy::too_many_arguments)]
fn compile_loop_stmt(
    loop_body: &[HirStmt],
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
    struct_depth: usize,
) -> Result<(), WasmCompileError> {
    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        return_type,
        false,
        repeat_id,
        struct_depth,
    )?;

    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    control_stack.pop();
    func.instruction(&Instruction::End);
    control_stack.pop();
    func.instruction(&Instruction::End);

    Ok(())
}

/// Compiles a counted `repeat count [as i] { body }` loop.
#[allow(clippy::too_many_arguments)]
fn compile_repeat_stmt(
    count: &HirExpr,
    maybe_index: Option<&str>,
    loop_body: &[HirStmt],
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
    struct_depth: usize,
) -> Result<(), WasmCompileError> {
    *repeat_id += 1;
    let id = *repeat_id;
    let limit_slot = locals[&format!("__repeat_limit_{id}")].0;
    let idx_slot = locals[&format!("__repeat_idx_{id}")].0;

    let count_ty = compile_expr(
        count,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        struct_depth,
    )?;
    coerce_type(func, count_ty, WasmType::I64);
    func.instruction(&Instruction::LocalSet(limit_slot));

    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(idx_slot));

    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    func.instruction(&Instruction::LocalGet(idx_slot));
    func.instruction(&Instruction::LocalGet(limit_slot));
    func.instruction(&Instruction::I64LtS);
    func.instruction(&Instruction::I32Eqz);
    let break_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopBreak)
        .unwrap() as u32;
    func.instruction(&Instruction::BrIf(break_depth));

    if let Some(name) = maybe_index {
        let user_idx_slot = locals[name].0;
        func.instruction(&Instruction::LocalGet(idx_slot));
        func.instruction(&Instruction::LocalSet(user_idx_slot));
    }

    control_stack.push(ControlFrame::RepeatStep);
    func.instruction(&Instruction::Block(BlockType::Empty));
    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        structs,
        static_strings,
        alloc_func_idx,
        return_type,
        false,
        repeat_id,
        struct_depth,
    )?;
    control_stack.pop();
    func.instruction(&Instruction::End);

    func.instruction(&Instruction::LocalGet(idx_slot));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(idx_slot));

    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    control_stack.pop();
    func.instruction(&Instruction::End);
    control_stack.pop();
    func.instruction(&Instruction::End);

    Ok(())
}

/// Helper to get span from statement.
fn program_stmt_span(stmt: &HirStmt) -> SourceSpan {
    match stmt {
        HirStmt::Let(_, _, span)
        | HirStmt::Var(_, _, span)
        | HirStmt::Assign(_, _, span)
        | HirStmt::CompoundAssign(_, _, _, span)
        | HirStmt::Loop(_, span)
        | HirStmt::While(_, _, span)
        | HirStmt::Repeat(_, _, _, span)
        | HirStmt::Each(_, _, _, span)
        | HirStmt::Break(span)
        | HirStmt::Continue(span)
        | HirStmt::Return(_, span)
        | HirStmt::Fail(_, span)
        | HirStmt::AwaitDo(_, span) => *span,
        HirStmt::If(s) => s.span,
        HirStmt::Match(s) => s.span,
        HirStmt::Attempt(s) => s.span,
        HirStmt::FnDecl(f) => f.span,
        HirStmt::Expr(e) => e.span(),
    }
}

/// Compresses a slice of local types into run-length encoded `(count, ValType)` pairs.
fn compress_locals(types: &[WasmType]) -> Vec<(u32, ValType)> {
    let mut compressed = Vec::new();
    for &ty in types {
        let val_ty: ValType = ty.into();
        if let Some((count, last_ty)) = compressed.last_mut() {
            if *last_ty == val_ty {
                *count += 1;
                continue;
            }
        }
        compressed.push((1, val_ty));
    }
    compressed
}

/// Compiles an expression and emits its WebAssembly instructions, returning its result type.
#[allow(clippy::too_many_arguments)]
fn compile_expr(
    expr: &HirExpr,
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    structs: &HashMap<String, StructLayout>,
    static_strings: &HashMap<String, i32>,
    alloc_func_idx: u32,
    struct_depth: usize,
) -> Result<WasmType, WasmCompileError> {
    match expr {
        HirExpr::Literal(lit, span) => match lit {
            Literal::Int(raw) => {
                let val =
                    parse_int_literal(raw).ok_or_else(|| WasmCompileError::InvalidLiteral {
                        message: format!("invalid integer literal `{raw}`"),
                        span: *span,
                    })?;
                func.instruction(&Instruction::I64Const(val));
                Ok(WasmType::I64)
            }
            Literal::Float(raw) => {
                let val =
                    parse_float_literal(raw).ok_or_else(|| WasmCompileError::InvalidLiteral {
                        message: format!("invalid float literal `{raw}`"),
                        span: *span,
                    })?;
                func.instruction(&Instruction::F64Const(val.into()));
                Ok(WasmType::F64)
            }
            Literal::Bool(b) => {
                func.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                Ok(WasmType::I32)
            }
            Literal::String(text, _) => {
                if let Some(&offset) = static_strings.get(text) {
                    func.instruction(&Instruction::I32Const(offset));
                    Ok(WasmType::I32)
                } else {
                    Err(WasmCompileError::InvalidLiteral {
                        message: format!("unregistered static string `{text}`"),
                        span: *span,
                    })
                }
            }
            Literal::None => {
                func.instruction(&Instruction::I64Const(0));
                Ok(WasmType::I64)
            }
        },
        HirExpr::Identifier(name, span) => {
            if let Some(&(idx, ty, _)) = locals.get(name) {
                func.instruction(&Instruction::LocalGet(idx));
                Ok(ty)
            } else {
                Err(WasmCompileError::UnknownVariable {
                    name: name.clone(),
                    span: *span,
                })
            }
        }
        HirExpr::Unary(op, inner, span) => {
            let inner_ty = compile_expr(
                inner,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                struct_depth,
            )?;
            match op {
                UnaryOp::Neg => match inner_ty {
                    WasmType::I64 => {
                        func.instruction(&Instruction::I64Const(-1));
                        func.instruction(&Instruction::I64Mul);
                        Ok(WasmType::I64)
                    }
                    WasmType::F64 => {
                        func.instruction(&Instruction::F64Neg);
                        Ok(WasmType::F64)
                    }
                    WasmType::I32 => {
                        func.instruction(&Instruction::I32Const(-1));
                        func.instruction(&Instruction::I32Mul);
                        Ok(WasmType::I32)
                    }
                },
                UnaryOp::Pos => Ok(inner_ty),
                UnaryOp::Not => match inner_ty {
                    WasmType::I32 => {
                        func.instruction(&Instruction::I32Eqz);
                        Ok(WasmType::I32)
                    }
                    WasmType::I64 => {
                        func.instruction(&Instruction::I64Eqz);
                        Ok(WasmType::I32)
                    }
                    WasmType::F64 => Err(WasmCompileError::TypeMismatch {
                        expected: "Int or Bool".into(),
                        found: "Float".into(),
                        span: *span,
                    }),
                },
            }
        }
        HirExpr::Binary(op, left, right, span) => {
            let left_ty = infer_expr_type(left, locals, functions, structs)?;
            let right_ty = infer_expr_type(right, locals, functions, structs)?;

            match op {
                BinaryOp::Div => {
                    let l_ty = compile_expr(
                        left,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, l_ty, WasmType::F64);
                    let r_ty = compile_expr(
                        right,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, WasmType::F64);
                    func.instruction(&Instruction::F64Div);
                    Ok(WasmType::F64)
                }
                BinaryOp::IntDiv => {
                    let l_ty = compile_expr(
                        left,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, l_ty, WasmType::I64);
                    let r_ty = compile_expr(
                        right,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, WasmType::I64);
                    func.instruction(&Instruction::I64DivS);
                    Ok(WasmType::I64)
                }
                BinaryOp::Mod => {
                    let l_ty = compile_expr(
                        left,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, l_ty, WasmType::I64);
                    let r_ty = compile_expr(
                        right,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, WasmType::I64);
                    func.instruction(&Instruction::I64RemS);
                    Ok(WasmType::I64)
                }
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                    let target_ty = if left_ty == WasmType::F64 || right_ty == WasmType::F64 {
                        WasmType::F64
                    } else {
                        WasmType::I64
                    };
                    let l_ty = compile_expr(
                        left,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, l_ty, target_ty);
                    let r_ty = compile_expr(
                        right,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, target_ty);

                    match (op, target_ty) {
                        (BinaryOp::Add, WasmType::F64) => func.instruction(&Instruction::F64Add),
                        (BinaryOp::Add, _) => func.instruction(&Instruction::I64Add),
                        (BinaryOp::Sub, WasmType::F64) => func.instruction(&Instruction::F64Sub),
                        (BinaryOp::Sub, _) => func.instruction(&Instruction::I64Sub),
                        (BinaryOp::Mul, WasmType::F64) => func.instruction(&Instruction::F64Mul),
                        (BinaryOp::Mul, _) => func.instruction(&Instruction::I64Mul),
                        _ => unreachable!(),
                    };
                    Ok(target_ty)
                }
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => {
                    let compare_ty = if left_ty == WasmType::F64 || right_ty == WasmType::F64 {
                        WasmType::F64
                    } else {
                        WasmType::I64
                    };
                    let l_ty = compile_expr(
                        left,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, l_ty, compare_ty);
                    let r_ty = compile_expr(
                        right,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, compare_ty);

                    match (op, compare_ty) {
                        (BinaryOp::Equal, WasmType::F64) => func.instruction(&Instruction::F64Eq),
                        (BinaryOp::Equal, _) => func.instruction(&Instruction::I64Eq),
                        (BinaryOp::NotEqual, WasmType::F64) => {
                            func.instruction(&Instruction::F64Ne)
                        }
                        (BinaryOp::NotEqual, _) => func.instruction(&Instruction::I64Ne),
                        (BinaryOp::Less, WasmType::F64) => func.instruction(&Instruction::F64Lt),
                        (BinaryOp::Less, _) => func.instruction(&Instruction::I64LtS),
                        (BinaryOp::LessEqual, WasmType::F64) => {
                            func.instruction(&Instruction::F64Le)
                        }
                        (BinaryOp::LessEqual, _) => func.instruction(&Instruction::I64LeS),
                        (BinaryOp::Greater, WasmType::F64) => func.instruction(&Instruction::F64Gt),
                        (BinaryOp::Greater, _) => func.instruction(&Instruction::I64GtS),
                        (BinaryOp::GreaterEqual, WasmType::F64) => {
                            func.instruction(&Instruction::F64Ge)
                        }
                        (BinaryOp::GreaterEqual, _) => func.instruction(&Instruction::I64GeS),
                        _ => unreachable!(),
                    };
                    Ok(WasmType::I32)
                }
                other => Err(WasmCompileError::UnsupportedExpr {
                    message: format!("operator `{other:?}` is not yet supported in Wasm backend"),
                    span: *span,
                }),
            }
        }
        HirExpr::If(cond, then_expr, else_expr, _) => {
            let res_ty = infer_expr_type(then_expr, locals, functions, structs)?;
            let cond_ty = compile_expr(
                cond,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                struct_depth,
            )?;
            coerce_to_bool(func, cond_ty);

            control_stack.push(ControlFrame::Block);
            func.instruction(&Instruction::If(BlockType::Result(res_ty.into())));

            let t_ty = compile_expr(
                then_expr,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                struct_depth,
            )?;
            coerce_type(func, t_ty, res_ty);

            func.instruction(&Instruction::Else);

            let e_ty = compile_expr(
                else_expr,
                func,
                locals,
                functions,
                control_stack,
                structs,
                static_strings,
                alloc_func_idx,
                struct_depth,
            )?;
            coerce_type(func, e_ty, res_ty);

            control_stack.pop();
            func.instruction(&Instruction::End);

            Ok(res_ty)
        }
        HirExpr::Call(callee, args, span) => {
            if let HirExpr::Identifier(func_name, _) = &**callee {
                if let Some(&(func_idx, _, ref fn_type)) = functions.get(func_name) {
                    if args.len() != fn_type.params.len() {
                        return Err(WasmCompileError::TypeMismatch {
                            expected: format!("{} arguments", fn_type.params.len()),
                            found: format!("{} arguments", args.len()),
                            span: *span,
                        });
                    }
                    for (arg, &expected_ty) in args.iter().zip(fn_type.params.iter()) {
                        let actual_ty = compile_expr(
                            &arg.value,
                            func,
                            locals,
                            functions,
                            control_stack,
                            structs,
                            static_strings,
                            alloc_func_idx,
                            struct_depth,
                        )?;
                        coerce_type(func, actual_ty, expected_ty);
                    }
                    func.instruction(&Instruction::Call(func_idx));
                    if let Some(&ret) = fn_type.results.first() {
                        Ok(ret)
                    } else {
                        Ok(WasmType::I32)
                    }
                } else {
                    Err(WasmCompileError::UnknownVariable {
                        name: func_name.clone(),
                        span: *span,
                    })
                }
            } else {
                Err(WasmCompileError::UnsupportedExpr {
                    message: "indirect calls are scheduled for Milestone 4".into(),
                    span: *span,
                })
            }
        }
        HirExpr::Construct(type_name, fields, span) => {
            if let Some(struct_layout) = structs.get(type_name) {
                let temp_name = format!("__struct_temp_{}", struct_depth.min(7));
                let struct_temp = locals[&temp_name].0;

                // Allocate memory block for instance
                func.instruction(&Instruction::I32Const(struct_layout.size as i32));
                func.instruction(&Instruction::Call(alloc_func_idx));
                func.instruction(&Instruction::LocalSet(struct_temp));

                // Initialize fields
                for (i, (maybe_name, field_expr)) in fields.iter().enumerate() {
                    let (field_offset, field_ty) = if let Some(name) = maybe_name {
                        struct_layout
                            .fields
                            .get(name)
                            .copied()
                            .unwrap_or(((i * 8) as u32, WasmType::I64))
                    } else {
                        // Positional initialization
                        ((i * 8) as u32, WasmType::I64)
                    };

                    func.instruction(&Instruction::LocalGet(struct_temp));
                    let val_ty = compile_expr(
                        field_expr,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth + 1,
                    )?;
                    coerce_type(func, val_ty, field_ty);
                    match field_ty {
                        WasmType::F64 => {
                            func.instruction(&Instruction::F64Store(MemArg {
                                offset: field_offset as u64,
                                align: 3,
                                memory_index: 0,
                            }));
                        }
                        WasmType::I32 => {
                            func.instruction(&Instruction::I32Store(MemArg {
                                offset: field_offset as u64,
                                align: 2,
                                memory_index: 0,
                            }));
                        }
                        WasmType::I64 => {
                            func.instruction(&Instruction::I64Store(MemArg {
                                offset: field_offset as u64,
                                align: 3,
                                memory_index: 0,
                            }));
                        }
                    }
                }

                // Return instance pointer
                func.instruction(&Instruction::LocalGet(struct_temp));
                Ok(WasmType::I32)
            } else {
                Err(WasmCompileError::UnsupportedExpr {
                    message: format!("unknown struct type `{type_name}`"),
                    span: *span,
                })
            }
        }
        HirExpr::Dot(receiver, field_name, span) => {
            let receiver_struct_name = match &**receiver {
                HirExpr::Identifier(name, _) => {
                    locals.get(name).and_then(|(_, _, kind)| match kind {
                        LocalKind::Struct(s_name) => Some(s_name.as_str()),
                        _ => None,
                    })
                }
                _ => None,
            };

            let is_string_receiver = match &**receiver {
                HirExpr::Identifier(name, _) => locals
                    .get(name)
                    .map(|(_, _, kind)| *kind == LocalKind::String)
                    .unwrap_or(false),
                HirExpr::Literal(Literal::String(..), _) => true,
                _ => false,
            };

            if field_name == "len"
                && (is_string_receiver
                    || (receiver_struct_name.is_none()
                        && !structs.values().any(|s| s.fields.contains_key("len"))))
            {
                let r_ty = compile_expr(
                    receiver,
                    func,
                    locals,
                    functions,
                    control_stack,
                    structs,
                    static_strings,
                    alloc_func_idx,
                    struct_depth,
                )?;
                coerce_type(func, r_ty, WasmType::I32);
                func.instruction(&Instruction::I32Load(MemArg {
                    offset: 0,
                    align: 2,
                    memory_index: 0,
                }));
                func.instruction(&Instruction::I64ExtendI32U);
                Ok(WasmType::I64)
            } else {
                let field_info = if let Some(s_name) = receiver_struct_name {
                    structs
                        .get(s_name)
                        .and_then(|s| s.fields.get(field_name).copied())
                } else {
                    structs
                        .values()
                        .find_map(|s| s.fields.get(field_name).copied())
                };

                if let Some((field_offset, ty)) = field_info {
                    let r_ty = compile_expr(
                        receiver,
                        func,
                        locals,
                        functions,
                        control_stack,
                        structs,
                        static_strings,
                        alloc_func_idx,
                        struct_depth,
                    )?;
                    coerce_type(func, r_ty, WasmType::I32);
                    match ty {
                        WasmType::F64 => {
                            func.instruction(&Instruction::F64Load(MemArg {
                                offset: field_offset as u64,
                                align: 3,
                                memory_index: 0,
                            }));
                            Ok(WasmType::F64)
                        }
                        WasmType::I32 => {
                            func.instruction(&Instruction::I32Load(MemArg {
                                offset: field_offset as u64,
                                align: 2,
                                memory_index: 0,
                            }));
                            Ok(WasmType::I32)
                        }
                        WasmType::I64 => {
                            func.instruction(&Instruction::I64Load(MemArg {
                                offset: field_offset as u64,
                                align: 3,
                                memory_index: 0,
                            }));
                            Ok(WasmType::I64)
                        }
                    }
                } else {
                    Err(WasmCompileError::UnsupportedExpr {
                        message: format!("unknown field `{field_name}` on struct"),
                        span: *span,
                    })
                }
            }
        }
        other => Err(WasmCompileError::UnsupportedExpr {
            message: format!("expression `{other:?}` is not yet supported in Wasm backend"),
            span: other.span(),
        }),
    }
}

/// Emits an arithmetic binary operation on top of stack.
fn emit_binary_op(
    func: &mut Function,
    op: BinaryOp,
    target_ty: WasmType,
    expr_ty: WasmType,
    span: SourceSpan,
) -> Result<(), WasmCompileError> {
    coerce_type(func, expr_ty, target_ty);
    match (op, target_ty) {
        (BinaryOp::Add, WasmType::F64) => func.instruction(&Instruction::F64Add),
        (BinaryOp::Add, _) => func.instruction(&Instruction::I64Add),
        (BinaryOp::Sub, WasmType::F64) => func.instruction(&Instruction::F64Sub),
        (BinaryOp::Sub, _) => func.instruction(&Instruction::I64Sub),
        (BinaryOp::Mul, WasmType::F64) => func.instruction(&Instruction::F64Mul),
        (BinaryOp::Mul, _) => func.instruction(&Instruction::I64Mul),
        (BinaryOp::IntDiv, _) => func.instruction(&Instruction::I64DivS),
        (BinaryOp::Div, _) => func.instruction(&Instruction::F64Div),
        (BinaryOp::Mod, _) => func.instruction(&Instruction::I64RemS),
        _ => {
            return Err(WasmCompileError::UnsupportedExpr {
                message: format!("compound assignment with `{op:?}` not supported"),
                span,
            });
        }
    };
    Ok(())
}

/// Coerces a value on top of the Wasm stack to boolean `i32` (0 or 1).
fn coerce_to_bool(func: &mut Function, ty: WasmType) {
    match ty {
        WasmType::I32 => {}
        WasmType::I64 => {
            func.instruction(&Instruction::I64Const(0));
            func.instruction(&Instruction::I64Ne);
        }
        WasmType::F64 => {
            func.instruction(&Instruction::F64Const(0.0.into()));
            func.instruction(&Instruction::F64Ne);
        }
    }
}

/// Coerces a value on top of the Wasm stack from `from` type to `to` type.
fn coerce_type(func: &mut Function, from: WasmType, to: WasmType) {
    if from == to {
        return;
    }
    match (from, to) {
        (WasmType::I64, WasmType::F64) => {
            func.instruction(&Instruction::F64ConvertI64S);
        }
        (WasmType::I32, WasmType::I64) => {
            func.instruction(&Instruction::I64ExtendI32U);
        }
        (WasmType::I32, WasmType::F64) => {
            func.instruction(&Instruction::F64ConvertI32S);
        }
        (WasmType::I64, WasmType::I32) => {
            func.instruction(&Instruction::I32WrapI64);
        }
        (WasmType::F64, WasmType::I64) => {
            func.instruction(&Instruction::I64TruncF64S);
        }
        (WasmType::F64, WasmType::I32) => {
            func.instruction(&Instruction::I32TruncF64S);
        }
        (WasmType::I32, WasmType::I32)
        | (WasmType::I64, WasmType::I64)
        | (WasmType::F64, WasmType::F64) => {}
    }
}

/// Infers the Wasm type of an HIR expression without emitting instructions.
fn infer_expr_type(
    expr: &HirExpr,
    locals: &HashMap<String, (u32, WasmType, LocalKind)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    structs: &HashMap<String, StructLayout>,
) -> Result<WasmType, WasmCompileError> {
    match expr {
        HirExpr::Literal(lit, _) => match lit {
            Literal::Int(_) => Ok(WasmType::I64),
            Literal::Float(_) => Ok(WasmType::F64),
            Literal::Bool(_) => Ok(WasmType::I32),
            Literal::String(..) => Ok(WasmType::I32),
            Literal::None => Ok(WasmType::I64),
        },
        HirExpr::Identifier(name, span) => {
            if let Some((_, ty, _)) = locals.get(name) {
                Ok(*ty)
            } else {
                Err(WasmCompileError::UnknownVariable {
                    name: name.clone(),
                    span: *span,
                })
            }
        }
        HirExpr::Unary(op, inner, _) => match op {
            UnaryOp::Neg | UnaryOp::Pos => infer_expr_type(inner, locals, functions, structs),
            UnaryOp::Not => Ok(WasmType::I32),
        },
        HirExpr::Binary(op, left, right, _) => match op {
            BinaryOp::Div => Ok(WasmType::F64),
            BinaryOp::IntDiv | BinaryOp::Mod => Ok(WasmType::I64),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => Ok(WasmType::I32),
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => {
                let left_ty = infer_expr_type(left, locals, functions, structs)?;
                let right_ty = infer_expr_type(right, locals, functions, structs)?;
                if left_ty == WasmType::F64 || right_ty == WasmType::F64 {
                    Ok(WasmType::F64)
                } else {
                    Ok(WasmType::I64)
                }
            }
            other => Err(WasmCompileError::UnsupportedExpr {
                message: format!("operator `{other:?}` is not supported in Wasm backend"),
                span: expr.span(),
            }),
        },
        HirExpr::If(_, then_expr, _, _) => infer_expr_type(then_expr, locals, functions, structs),
        HirExpr::Construct(..) => Ok(WasmType::I32),
        HirExpr::Dot(receiver, field_name, _) => {
            let receiver_struct_name = match &**receiver {
                HirExpr::Identifier(name, _) => {
                    locals.get(name).and_then(|(_, _, kind)| match kind {
                        LocalKind::Struct(s_name) => Some(s_name.as_str()),
                        _ => None,
                    })
                }
                _ => None,
            };
            if field_name == "len"
                && (receiver_struct_name.is_none()
                    && !structs.values().any(|s| s.fields.contains_key("len")))
            {
                Ok(WasmType::I64)
            } else {
                let field_info = if let Some(s_name) = receiver_struct_name {
                    structs
                        .get(s_name)
                        .and_then(|s| s.fields.get(field_name).copied())
                } else {
                    structs
                        .values()
                        .find_map(|s| s.fields.get(field_name).copied())
                };
                Ok(field_info.map(|(_, ty)| ty).unwrap_or(WasmType::I64))
            }
        }
        HirExpr::Call(callee, _, span) => {
            if let HirExpr::Identifier(func_name, _) = &**callee {
                if let Some((_, _, fn_type)) = functions.get(func_name) {
                    if let Some(&ret) = fn_type.results.first() {
                        Ok(ret)
                    } else {
                        Err(WasmCompileError::TypeMismatch {
                            expected: "value-returning function".to_string(),
                            found: "void function".to_string(),
                            span: *span,
                        })
                    }
                } else {
                    Err(WasmCompileError::UnknownVariable {
                        name: func_name.clone(),
                        span: *span,
                    })
                }
            } else {
                Err(WasmCompileError::UnsupportedExpr {
                    message: "indirect calls are scheduled for Milestone 4".to_string(),
                    span: *span,
                })
            }
        }
        other => Err(WasmCompileError::UnsupportedExpr {
            message: format!("expression `{other:?}` is not supported in Wasm backend"),
            span: other.span(),
        }),
    }
}
