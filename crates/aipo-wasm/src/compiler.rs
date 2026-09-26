//! WebAssembly compiler lowering Aipo HIR to standard Wasm binary modules (ADP-013).

use crate::emitter::WasmEmitter;
use crate::error::WasmCompileError;
use crate::types::{WasmFnType, WasmType};
use aipo_ast::{BinaryOp, Literal, UnaryOp};
use aipo_hir::{HirExpr, HirFunctionDecl, HirIfStmt, HirItem, HirParam, HirProgram, HirStmt};
use aipo_lexer::{parse_float_literal, parse_int_literal};
use aipo_source::SourceSpan;
use std::collections::HashMap;
use wasm_encoder::{BlockType, Function, Instruction, ValType};

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
                let ret =
                    resolve_return_type(&func.return_type, &func.params, &func.body, &functions);
                let fn_type = WasmFnType::new(params, ret.into_iter().collect());
                let type_idx = emitter.add_type(fn_type.clone());
                let func_idx = functions.len() as u32;
                functions.insert(func.name.clone(), (func_idx, type_idx, fn_type));
                func_decls.push(func);
            }
            HirItem::Export(_) | HirItem::Import(_) => {
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
        let ret = infer_body_return_type(&program.statements, &mut top_locals, &functions);
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
        )?;
        let assigned_idx = emitter.add_function(*type_idx, compiled_fn);
        emitter.export_function(&func.name, assigned_idx);
    }

    // Compile top-level entrypoint if present
    if let Some((_, type_idx, ret)) = top_level_info {
        let compiled_fn =
            compile_function_body("__top_level__", &[], ret, &program.statements, &functions)?;
        let assigned_idx = emitter.add_function(type_idx, compiled_fn);
        emitter.export_function("__top_level__", assigned_idx);
    }

    Ok(emitter.finish())
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
) -> Option<WasmType> {
    if let Some(ty) = annot {
        match ty.name.as_str() {
            "Float" => Some(WasmType::F64),
            "Bool" => Some(WasmType::I32),
            "None" => None,
            _ => Some(WasmType::I64),
        }
    } else {
        let mut locals = HashMap::new();
        for (i, param) in params.iter().enumerate() {
            locals.insert(param.name.clone(), (i as u32, param_wasm_type(param)));
        }
        infer_body_return_type(body, &mut locals, functions)
    }
}

/// Scans statements to deduce the return type of a block or function body.
fn infer_body_return_type(
    body: &[HirStmt],
    locals: &mut HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
) -> Option<WasmType> {
    let mut has_return = false;
    for stmt in body {
        match stmt {
            HirStmt::Let(name, expr, _) | HirStmt::Var(name, expr, _) => {
                let ty = infer_expr_type(expr, locals, functions).unwrap_or(WasmType::I64);
                let idx = locals.len() as u32;
                locals.insert(name.clone(), (idx, ty));
            }
            HirStmt::Return(Some(expr), _) => {
                has_return = true;
                if let Ok(ty) = infer_expr_type(expr, locals, functions) {
                    return Some(ty);
                }
            }
            HirStmt::Return(None, _) => {
                return None;
            }
            HirStmt::If(s) => {
                if let Some(ty) = infer_body_return_type(&s.then_branch, locals, functions) {
                    return Some(ty);
                }
                for (_, elif_body) in &s.elif_branches {
                    if let Some(ty) = infer_body_return_type(elif_body, locals, functions) {
                        return Some(ty);
                    }
                }
                if let Some(else_branch) = &s.else_branch {
                    if let Some(ty) = infer_body_return_type(else_branch, locals, functions) {
                        return Some(ty);
                    }
                }
            }
            HirStmt::While(_, loop_body, _)
            | HirStmt::Loop(loop_body, _)
            | HirStmt::Repeat(_, _, loop_body, _) => {
                if let Some(ty) = infer_body_return_type(loop_body, locals, functions) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }
    // If the last statement is an expression statement, its type is the implicit return
    if let Some(HirStmt::Expr(expr)) = body.last() {
        if let Ok(ty) = infer_expr_type(expr, locals, functions) {
            return Some(ty);
        }
    }
    if has_return {
        Some(WasmType::I64)
    } else {
        None
    }
}

/// Recursively scans statements to register all declared locals.
fn pre_scan_stmts(
    stmts: &[HirStmt],
    params_count: usize,
    locals: &mut HashMap<String, (u32, WasmType)>,
    declared_locals: &mut Vec<WasmType>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    for stmt in stmts {
        match stmt {
            HirStmt::Let(name, init_expr, _) | HirStmt::Var(name, init_expr, _) => {
                let ty = infer_expr_type(init_expr, locals, functions).unwrap_or(WasmType::I64);
                let idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(ty);
                locals.insert(name.clone(), (idx, ty));
            }
            HirStmt::If(s) => {
                pre_scan_stmts(
                    &s.then_branch,
                    params_count,
                    locals,
                    declared_locals,
                    functions,
                    repeat_id,
                )?;
                for (_, elif_body) in &s.elif_branches {
                    pre_scan_stmts(
                        elif_body,
                        params_count,
                        locals,
                        declared_locals,
                        functions,
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
                    repeat_id,
                )?;
            }
            HirStmt::Repeat(_, maybe_index, body, _) => {
                *repeat_id += 1;
                let id = *repeat_id;
                // repeat_limit slot
                let limit_idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(WasmType::I64);
                locals.insert(format!("__repeat_limit_{id}"), (limit_idx, WasmType::I64));
                // repeat_idx slot
                let idx_idx = (params_count + declared_locals.len()) as u32;
                declared_locals.push(WasmType::I64);
                locals.insert(format!("__repeat_idx_{id}"), (idx_idx, WasmType::I64));
                // user index variable if specified
                if let Some(name) = maybe_index {
                    let user_idx = (params_count + declared_locals.len()) as u32;
                    declared_locals.push(WasmType::I64);
                    locals.insert(name.clone(), (user_idx, WasmType::I64));
                }
                pre_scan_stmts(
                    body,
                    params_count,
                    locals,
                    declared_locals,
                    functions,
                    repeat_id,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Compiles a single function's body into a WebAssembly `Function`.
fn compile_function_body(
    _func_name: &str,
    params: &[HirParam],
    return_type: Option<WasmType>,
    body: &[HirStmt],
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
) -> Result<Function, WasmCompileError> {
    let mut locals: HashMap<String, (u32, WasmType)> = HashMap::new();
    let mut declared_locals: Vec<WasmType> = Vec::new();

    // Map parameters to locals 0..params.len()
    for (i, param) in params.iter().enumerate() {
        let ty = param_wasm_type(param);
        locals.insert(param.name.clone(), (i as u32, ty));
    }

    // Pre-scan pass to register all local variable declarations (let/var/repeat)
    let mut pre_scan_repeat_id = 0;
    pre_scan_stmts(
        body,
        params.len(),
        &mut locals,
        &mut declared_locals,
        functions,
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
        return_type,
        true,
        &mut emit_repeat_id,
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
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    return_type: Option<WasmType>,
    is_top_level: bool,
    repeat_id: &mut u32,
) -> Result<bool, WasmCompileError> {
    let mut terminated = false;
    let stmt_count = stmts.len();

    for (idx, stmt) in stmts.iter().enumerate() {
        let is_last = idx + 1 == stmt_count;
        match stmt {
            HirStmt::Let(name, init_expr, _) | HirStmt::Var(name, init_expr, _) => {
                let (local_idx, local_ty) = locals[name];
                let expr_ty = compile_expr(init_expr, func, locals, functions, control_stack)?;
                coerce_type(func, expr_ty, local_ty);
                func.instruction(&Instruction::LocalSet(local_idx));
            }
            HirStmt::Assign(target, expr, span) => {
                if let HirExpr::Identifier(name, _) = target {
                    if let Some(&(local_idx, target_ty)) = locals.get(name) {
                        let expr_ty = compile_expr(expr, func, locals, functions, control_stack)?;
                        coerce_type(func, expr_ty, target_ty);
                        func.instruction(&Instruction::LocalSet(local_idx));
                    } else {
                        return Err(WasmCompileError::UnknownVariable {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                } else {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "only identifier assignment is supported in Wasm backend".into(),
                        span: *span,
                    });
                }
            }
            HirStmt::CompoundAssign(op, target, expr, span) => {
                if let HirExpr::Identifier(name, _) = target {
                    if let Some(&(local_idx, target_ty)) = locals.get(name) {
                        func.instruction(&Instruction::LocalGet(local_idx));
                        let expr_ty = compile_expr(expr, func, locals, functions, control_stack)?;
                        emit_binary_op(func, *op, target_ty, expr_ty, *span)?;
                        func.instruction(&Instruction::LocalSet(local_idx));
                    } else {
                        return Err(WasmCompileError::UnknownVariable {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                } else {
                    return Err(WasmCompileError::UnsupportedStmt {
                        message: "only identifier compound assignment is supported in Wasm backend"
                            .into(),
                        span: *span,
                    });
                }
            }
            HirStmt::Return(maybe_expr, span) => {
                if let Some(expr) = maybe_expr {
                    let expr_ty = compile_expr(expr, func, locals, functions, control_stack)?;
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
                    return_type,
                    repeat_id,
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
                    return_type,
                    repeat_id,
                )?;
            }
            HirStmt::Loop(loop_body, _) => {
                compile_loop_stmt(
                    loop_body,
                    func,
                    locals,
                    functions,
                    control_stack,
                    return_type,
                    repeat_id,
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
                    return_type,
                    repeat_id,
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
                let expr_ty = compile_expr(expr, func, locals, functions, control_stack)?;
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
                    message: format!("statement `{other:?}` is scheduled for Milestone 3"),
                    span: program_stmt_span(other),
                });
            }
        }
    }

    Ok(terminated)
}

/// Compiles an `if ... elif ... else` statement block.
fn compile_if_stmt(
    if_stmt: &HirIfStmt,
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    let cond_ty = compile_expr(&if_stmt.condition, func, locals, functions, control_stack)?;
    coerce_to_bool(func, cond_ty);

    control_stack.push(ControlFrame::Block);
    func.instruction(&Instruction::If(BlockType::Empty));
    compile_stmts(
        &if_stmt.then_branch,
        func,
        locals,
        functions,
        control_stack,
        return_type,
        false,
        repeat_id,
    )?;

    let has_elifs = !if_stmt.elif_branches.is_empty();
    let has_else = if_stmt.else_branch.is_some();

    if has_elifs || has_else {
        func.instruction(&Instruction::Else);

        let mut elif_count = 0;
        for (elif_cond, elif_body) in &if_stmt.elif_branches {
            let e_cond_ty = compile_expr(elif_cond, func, locals, functions, control_stack)?;
            coerce_to_bool(func, e_cond_ty);

            control_stack.push(ControlFrame::Block);
            func.instruction(&Instruction::If(BlockType::Empty));
            compile_stmts(
                elif_body,
                func,
                locals,
                functions,
                control_stack,
                return_type,
                false,
                repeat_id,
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
                return_type,
                false,
                repeat_id,
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
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    // Outer block for forward break
    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    // Inner loop for backward continue
    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    // Evaluate condition
    let cond_ty = compile_expr(cond, func, locals, functions, control_stack)?;
    coerce_to_bool(func, cond_ty);
    func.instruction(&Instruction::I32Eqz);

    // If condition is false, break forward out of outer block
    let break_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopBreak)
        .unwrap() as u32;
    func.instruction(&Instruction::BrIf(break_depth));

    // Body
    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        return_type,
        false,
        repeat_id,
    )?;

    // Jump back to loop start
    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    // Close loop and block
    control_stack.pop();
    func.instruction(&Instruction::End);
    control_stack.pop();
    func.instruction(&Instruction::End);

    Ok(())
}

/// Compiles an unconditional `loop { body }` structure.
fn compile_loop_stmt(
    loop_body: &[HirStmt],
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    // Outer block for forward break
    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    // Inner loop for backward continue
    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    // Body
    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        return_type,
        false,
        repeat_id,
    )?;

    // Jump back to start
    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    // Close loop and block
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
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
    return_type: Option<WasmType>,
    repeat_id: &mut u32,
) -> Result<(), WasmCompileError> {
    *repeat_id += 1;
    let id = *repeat_id;
    let limit_slot = locals[&format!("__repeat_limit_{id}")].0;
    let idx_slot = locals[&format!("__repeat_idx_{id}")].0;

    // Evaluate count and store in limit_slot
    let count_ty = compile_expr(count, func, locals, functions, control_stack)?;
    coerce_type(func, count_ty, WasmType::I64);
    func.instruction(&Instruction::LocalSet(limit_slot));

    // Initialize repeat index to 0
    func.instruction(&Instruction::I64Const(0));
    func.instruction(&Instruction::LocalSet(idx_slot));

    // Outer block for break
    control_stack.push(ControlFrame::LoopBreak);
    func.instruction(&Instruction::Block(BlockType::Empty));

    // Inner loop for continue/iterations
    control_stack.push(ControlFrame::LoopContinue);
    func.instruction(&Instruction::Loop(BlockType::Empty));

    // Condition check: repeat_idx < repeat_limit
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

    // Update user index variable if specified
    if let Some(name) = maybe_index {
        let user_idx_slot = locals[name].0;
        func.instruction(&Instruction::LocalGet(idx_slot));
        func.instruction(&Instruction::LocalSet(user_idx_slot));
    }

    // Step block: `continue` jumps here to perform the increment
    control_stack.push(ControlFrame::RepeatStep);
    func.instruction(&Instruction::Block(BlockType::Empty));
    compile_stmts(
        loop_body,
        func,
        locals,
        functions,
        control_stack,
        return_type,
        false,
        repeat_id,
    )?;
    control_stack.pop();
    func.instruction(&Instruction::End);

    // Increment index: repeat_idx += 1
    func.instruction(&Instruction::LocalGet(idx_slot));
    func.instruction(&Instruction::I64Const(1));
    func.instruction(&Instruction::I64Add);
    func.instruction(&Instruction::LocalSet(idx_slot));

    // Repeat loop
    let loop_depth = control_stack
        .iter()
        .rev()
        .position(|f| *f == ControlFrame::LoopContinue)
        .unwrap() as u32;
    func.instruction(&Instruction::Br(loop_depth));

    // Close loop and block
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
fn compile_expr(
    expr: &HirExpr,
    func: &mut Function,
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
    control_stack: &mut Vec<ControlFrame>,
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
            other => Err(WasmCompileError::UnsupportedExpr {
                message: format!("literal {other:?} is not supported in Wasm arithmetic"),
                span: *span,
            }),
        },
        HirExpr::Identifier(name, span) => {
            if let Some(&(idx, ty)) = locals.get(name) {
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
            let inner_ty = compile_expr(inner, func, locals, functions, control_stack)?;
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
            let left_ty = infer_expr_type(left, locals, functions)?;
            let right_ty = infer_expr_type(right, locals, functions)?;

            match op {
                BinaryOp::Div => {
                    // In Aipo, `/` is always IEEE 754 float division
                    let l_ty = compile_expr(left, func, locals, functions, control_stack)?;
                    coerce_type(func, l_ty, WasmType::F64);
                    let r_ty = compile_expr(right, func, locals, functions, control_stack)?;
                    coerce_type(func, r_ty, WasmType::F64);
                    func.instruction(&Instruction::F64Div);
                    Ok(WasmType::F64)
                }
                BinaryOp::IntDiv => {
                    // Integer division `//` operates on signed 64-bit integers
                    let l_ty = compile_expr(left, func, locals, functions, control_stack)?;
                    coerce_type(func, l_ty, WasmType::I64);
                    let r_ty = compile_expr(right, func, locals, functions, control_stack)?;
                    coerce_type(func, r_ty, WasmType::I64);
                    func.instruction(&Instruction::I64DivS);
                    Ok(WasmType::I64)
                }
                BinaryOp::Mod => {
                    let l_ty = compile_expr(left, func, locals, functions, control_stack)?;
                    coerce_type(func, l_ty, WasmType::I64);
                    let r_ty = compile_expr(right, func, locals, functions, control_stack)?;
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
                    let l_ty = compile_expr(left, func, locals, functions, control_stack)?;
                    coerce_type(func, l_ty, target_ty);
                    let r_ty = compile_expr(right, func, locals, functions, control_stack)?;
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
                    let l_ty = compile_expr(left, func, locals, functions, control_stack)?;
                    coerce_type(func, l_ty, compare_ty);
                    let r_ty = compile_expr(right, func, locals, functions, control_stack)?;
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
            let res_ty = infer_expr_type(then_expr, locals, functions)?;
            let cond_ty = compile_expr(cond, func, locals, functions, control_stack)?;
            coerce_to_bool(func, cond_ty);

            control_stack.push(ControlFrame::Block);
            func.instruction(&Instruction::If(BlockType::Result(res_ty.into())));

            let t_ty = compile_expr(then_expr, func, locals, functions, control_stack)?;
            coerce_type(func, t_ty, res_ty);

            func.instruction(&Instruction::Else);

            let e_ty = compile_expr(else_expr, func, locals, functions, control_stack)?;
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
                        let actual_ty =
                            compile_expr(&arg.value, func, locals, functions, control_stack)?;
                        coerce_type(func, actual_ty, expected_ty);
                    }
                    func.instruction(&Instruction::Call(func_idx));
                    if let Some(&ret) = fn_type.results.first() {
                        Ok(ret)
                    } else {
                        // Void function call has no return value
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
        WasmType::I32 => {
            // Already i32 (boolean or integer flag)
        }
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
        _ => {}
    }
}

/// Infers the Wasm type of an HIR expression without emitting instructions.
fn infer_expr_type(
    expr: &HirExpr,
    locals: &HashMap<String, (u32, WasmType)>,
    functions: &HashMap<String, (u32, u32, WasmFnType)>,
) -> Result<WasmType, WasmCompileError> {
    match expr {
        HirExpr::Literal(lit, span) => match lit {
            Literal::Int(_) => Ok(WasmType::I64),
            Literal::Float(_) => Ok(WasmType::F64),
            Literal::Bool(_) => Ok(WasmType::I32),
            _ => Err(WasmCompileError::UnsupportedExpr {
                message: format!("literal {lit:?} not supported in arithmetic inference"),
                span: *span,
            }),
        },
        HirExpr::Identifier(name, span) => {
            if let Some((_, ty)) = locals.get(name) {
                Ok(*ty)
            } else {
                Err(WasmCompileError::UnknownVariable {
                    name: name.clone(),
                    span: *span,
                })
            }
        }
        HirExpr::Unary(op, inner, _) => match op {
            UnaryOp::Neg | UnaryOp::Pos => infer_expr_type(inner, locals, functions),
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
                let left_ty = infer_expr_type(left, locals, functions)?;
                let right_ty = infer_expr_type(right, locals, functions)?;
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
        HirExpr::If(_, then_expr, _, _) => infer_expr_type(then_expr, locals, functions),
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
