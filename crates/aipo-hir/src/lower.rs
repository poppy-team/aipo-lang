//! Lowering pass from AST to HIR.
//!
//! Desugars:
//! - Pipelines (`a |> f(b)` -> `f(a, b)`)
//! - Trailing blocks (`callee(args) do params ... end` -> `callee(args, fn(params) ... end)`)
//! - Binding destructuring (`let [a, b] = x` -> `let $t = x; let a = $t[0]; let b = $t[1]`)
//! - Struct destructuring (`let {x, y} = p` -> `let $t = p; let x = $t.x; let y = $t.y`)

use crate::hir::*;
use aipo_ast::*;
use aipo_source::SourceSpan;

/// State for the lowering pass, tracking synthetic variable counters.
#[derive(Default)]
pub struct LoweringContext {
    temp_counter: usize,
}

impl LoweringContext {
    /// Creates a new lowering context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Generates a fresh, unique synthetic variable name.
    fn fresh_temp(&mut self) -> String {
        let name = format!("__temp_{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    /// Lowers an AST `Program` into a `HirProgram`.
    pub fn lower_program(&mut self, program: Program) -> HirProgram {
        let items = program
            .items
            .into_iter()
            .map(|item| self.lower_item(item))
            .collect();

        let mut statements = Vec::new();
        for stmt in program.statements {
            self.lower_stmt(stmt, &mut statements);
        }

        HirProgram {
            items,
            statements,
            span: program.span,
        }
    }

    /// Lowers a structural module item.
    pub fn lower_item(&mut self, item: Item) -> HirItem {
        match item {
            Item::Fn(f) => HirItem::Fn(self.lower_function_decl(f)),
            Item::Struct(s) => HirItem::Struct(self.lower_struct_decl(s)),
            Item::Impl(i) => HirItem::Impl(self.lower_impl_block(i)),
            Item::Interface(i) => HirItem::Interface(self.lower_interface_decl(i)),
            Item::Satisfy(s) => HirItem::Satisfy(self.lower_satisfy_decl(s)),
            Item::Import(i) => HirItem::Import(self.lower_import_decl(i)),
            Item::Export(e) => HirItem::Export(self.lower_export_decl(e)),
        }
    }

    fn lower_function_decl(&mut self, f: FunctionDecl) -> HirFunctionDecl {
        let params = f.params.into_iter().map(|p| self.lower_param(p)).collect();
        let mut body = Vec::new();
        for stmt in f.body {
            self.lower_stmt(stmt, &mut body);
        }
        HirFunctionDecl {
            name: f.name.name,
            params,
            return_type: f.return_type,
            body,
            span: f.span,
        }
    }

    fn lower_param(&mut self, p: Param) -> HirParam {
        let is_self = p.name.name == "self";
        HirParam {
            name: p.name.name,
            is_mut: p.is_mutable,
            is_self,
            type_annotation: p.type_annotation,
            default: p.default.map(|d| self.lower_expr(d)),
            span: p.span,
        }
    }

    fn lower_struct_decl(&mut self, s: StructDecl) -> HirStructDecl {
        let fields = s
            .fields
            .into_iter()
            .map(|f| HirStructField {
                name: f.name.name,
                is_fixed: f.is_fixed,
                default: f.default.map(|d| self.lower_expr(d)),
                span: f.span,
            })
            .collect();
        HirStructDecl {
            name: s.name.name,
            fields,
            span: s.span,
        }
    }

    fn lower_impl_block(&mut self, i: ImplBlock) -> HirImplBlock {
        let init = i.init.map(|init_fn| self.lower_function_decl(init_fn));
        let invariant = i.invariant.map(|hook| self.lower_invariant_hook(hook));
        let methods = i
            .methods
            .into_iter()
            .map(|m| self.lower_function_decl(m))
            .collect();
        HirImplBlock {
            target: i.target.name,
            init,
            invariant,
            methods,
            span: i.span,
        }
    }

    /// Lowers `invariant()` into a `self`-taking predicate function.
    ///
    /// Canon says the hook's lines are conceptually conditions combined by `and` and that the
    /// hook does not take parameters besides a read-only `self`. Representing it as a function
    /// that returns the combined condition keeps one execution path for the check: the runtime
    /// calls it like any other predicate instead of needing a second expression evaluator.
    fn lower_invariant_hook(&mut self, hook: InvariantHook) -> HirFunctionDecl {
        let self_param = HirParam {
            name: "self".to_string(),
            is_mut: false,
            is_self: true,
            type_annotation: None,
            default: None,
            span: hook.span,
        };

        let mut conditions = hook.conditions.into_iter();
        let combined = match conditions.next() {
            Some(first) => {
                let mut acc = self.lower_expr(first);
                for next in conditions {
                    let span = acc.span();
                    let right = self.lower_expr(next);
                    acc = HirExpr::Binary(BinaryOp::And, Box::new(acc), Box::new(right), span);
                }
                acc
            }
            // An empty hook is vacuously true rather than a compile-time error: canon says a
            // hook holds "one or more" conditions, but a hook that validates nothing must not
            // silently reject every construction.
            None => HirExpr::Literal(Literal::Bool(true), hook.span),
        };

        HirFunctionDecl {
            name: "invariant".to_string(),
            params: vec![self_param],
            return_type: None,
            body: vec![HirStmt::Return(Some(combined), hook.span)],
            span: hook.span,
        }
    }

    fn lower_interface_decl(&mut self, i: InterfaceDecl) -> HirInterfaceDecl {
        let methods = i
            .methods
            .into_iter()
            .map(|m| self.lower_function_decl(m))
            .collect();
        HirInterfaceDecl {
            name: i.name.name,
            methods,
            span: i.span,
        }
    }

    fn lower_satisfy_decl(&mut self, s: SatisfyDecl) -> HirSatisfyDecl {
        HirSatisfyDecl {
            target: s.target.name,
            interfaces: s.interfaces.into_iter().map(|i| i.name).collect(),
            span: s.span,
        }
    }

    fn lower_import_decl(&mut self, i: ImportDecl) -> HirImportDecl {
        HirImportDecl {
            module_name: i.module_name.name,
            alias: i.alias.map(|a| a.name),
            names: i.names.into_iter().map(|n| n.name).collect(),
            span: i.span,
        }
    }

    fn lower_export_decl(&mut self, e: ExportDecl) -> HirExportDecl {
        HirExportDecl {
            names: e.names.into_iter().map(|n| n.name).collect(),
            span: e.span,
        }
    }

    /// Lowers an AST statement, potentially emitting multiple HIR statements for destructuring.
    pub fn lower_stmt(&mut self, stmt: Stmt, out: &mut Vec<HirStmt>) {
        match stmt {
            Stmt::Let(pattern, expr, span) => {
                let lowered_expr = self.lower_expr(expr);
                self.lower_binding_pattern(pattern, lowered_expr, false, span, out);
            }
            Stmt::Var(pattern, expr, span) => {
                let lowered_expr = self.lower_expr(expr);
                self.lower_binding_pattern(pattern, lowered_expr, true, span, out);
            }
            Stmt::Assign(target, value, span) => {
                out.push(HirStmt::Assign(
                    self.lower_expr(target),
                    self.lower_expr(value),
                    span,
                ));
            }
            Stmt::CompoundAssign(op, target, value, span) => {
                out.push(HirStmt::CompoundAssign(
                    op,
                    self.lower_expr(target),
                    self.lower_expr(value),
                    span,
                ));
            }
            Stmt::If(s) => {
                let condition = self.lower_expr(s.condition);
                let mut then_branch = Vec::new();
                for st in s.then_branch {
                    self.lower_stmt(st, &mut then_branch);
                }
                let mut elif_branches = Vec::new();
                for (c, stmts) in s.elif_branches {
                    let mut b = Vec::new();
                    for st in stmts {
                        self.lower_stmt(st, &mut b);
                    }
                    elif_branches.push((self.lower_expr(c), b));
                }
                let else_branch = s.else_branch.map(|stmts| {
                    let mut b = Vec::new();
                    for st in stmts {
                        self.lower_stmt(st, &mut b);
                    }
                    b
                });
                out.push(HirStmt::If(HirIfStmt {
                    condition,
                    then_branch,
                    elif_branches,
                    else_branch,
                    span: s.span,
                }));
            }
            Stmt::Match(s) => {
                let target = self.lower_expr(s.target);
                let mut when_arms = Vec::new();
                for (pats, body) in s.when_arms {
                    let patterns = pats.into_iter().map(|p| self.lower_expr(p)).collect();
                    let mut arm_body = Vec::new();
                    for st in body {
                        self.lower_stmt(st, &mut arm_body);
                    }
                    when_arms.push((patterns, arm_body));
                }
                let else_arm = s.else_arm.map(|body| {
                    let mut arm_body = Vec::new();
                    for st in body {
                        self.lower_stmt(st, &mut arm_body);
                    }
                    arm_body
                });
                out.push(HirStmt::Match(HirMatchStmt {
                    target,
                    when_arms,
                    else_arm,
                    span: s.span,
                }));
            }
            Stmt::Loop(body, span) => {
                let mut loop_body = Vec::new();
                for st in body {
                    self.lower_stmt(st, &mut loop_body);
                }
                out.push(HirStmt::Loop(loop_body, span));
            }
            Stmt::While(cond, body, span) => {
                let condition = self.lower_expr(cond);
                let mut while_body = Vec::new();
                for st in body {
                    self.lower_stmt(st, &mut while_body);
                }
                out.push(HirStmt::While(condition, while_body, span));
            }
            Stmt::Repeat(count, alias, body, span) => {
                let count_expr = self.lower_expr(count);
                let mut repeat_body = Vec::new();
                for st in body {
                    self.lower_stmt(st, &mut repeat_body);
                }
                out.push(HirStmt::Repeat(
                    count_expr,
                    alias.map(|a| a.name),
                    repeat_body,
                    span,
                ));
            }
            Stmt::Each(vars, iter, body, span) => {
                let iter_expr = self.lower_expr(iter);
                let mut each_body = Vec::new();
                for st in body {
                    self.lower_stmt(st, &mut each_body);
                }
                out.push(HirStmt::Each(
                    vars.into_iter().map(|v| v.name).collect(),
                    iter_expr,
                    each_body,
                    span,
                ));
            }
            Stmt::Break(span) => out.push(HirStmt::Break(span)),
            Stmt::Continue(span) => out.push(HirStmt::Continue(span)),
            Stmt::Return(expr, span) => {
                out.push(HirStmt::Return(expr.map(|e| self.lower_expr(e)), span))
            }
            Stmt::Fail(expr, span) => out.push(HirStmt::Fail(self.lower_expr(expr), span)),
            Stmt::Attempt(s) => {
                let mut body = Vec::new();
                for st in s.body {
                    self.lower_stmt(st, &mut body);
                }
                let mut handler = Vec::new();
                for st in s.failed_body {
                    self.lower_stmt(st, &mut handler);
                }
                out.push(HirStmt::Attempt(HirAttemptStmt {
                    body,
                    error_binding: s.err_binding.map(|e| e.name),
                    handler,
                    span: s.span,
                }));
            }
            Stmt::Fn(f) => {
                // The declared name survives lowering: the builder needs it to bind the local
                // function and to wire the self-capture that makes recursion work.
                out.push(HirStmt::FnDecl(self.lower_function_decl(f)));
            }
            Stmt::Expr(expr) => {
                out.push(HirStmt::Expr(self.lower_expr(expr)));
            }
        }
    }

    /// Desugars a binding pattern into one or more `HirStmt::Let` or `HirStmt::Var`.
    fn lower_binding_pattern(
        &mut self,
        pattern: BindingPattern,
        expr: HirExpr,
        is_var: bool,
        span: SourceSpan,
        out: &mut Vec<HirStmt>,
    ) {
        match pattern {
            BindingPattern::Ident(id) => {
                if is_var {
                    out.push(HirStmt::Var(id.name, expr, span));
                } else {
                    out.push(HirStmt::Let(id.name, expr, span));
                }
            }
            BindingPattern::Discard(_) => {
                out.push(HirStmt::Expr(expr));
            }
            BindingPattern::List(patterns, _) => {
                let temp = self.fresh_temp();
                out.push(HirStmt::Let(temp.clone(), expr, span));

                for (idx, pat) in patterns.into_iter().enumerate() {
                    let index_expr = HirExpr::Literal(Literal::Int(idx.to_string()), span);
                    let access = HirExpr::Index(
                        Box::new(HirExpr::Identifier(temp.clone(), span)),
                        Box::new(index_expr),
                        span,
                    );
                    self.lower_binding_pattern(pat, access, is_var, span, out);
                }
            }
            BindingPattern::Struct(fields, _) => {
                let temp = self.fresh_temp();
                out.push(HirStmt::Let(temp.clone(), expr, span));

                for field in fields {
                    let access = HirExpr::Dot(
                        Box::new(HirExpr::Identifier(temp.clone(), span)),
                        field.name.clone(),
                        field.span,
                    );
                    if is_var {
                        out.push(HirStmt::Var(field.name, access, field.span));
                    } else {
                        out.push(HirStmt::Let(field.name, access, field.span));
                    }
                }
            }
        }
    }

    /// Lowers an AST expression into an HIR expression, desugaring pipeline and trailing blocks.
    pub fn lower_expr(&mut self, expr: Expr) -> HirExpr {
        match expr {
            Expr::Literal(lit, span) => HirExpr::Literal(lit, span),
            Expr::Identifier(id) => HirExpr::Identifier(id.name, id.span),
            Expr::Unary(op, inner, span) => {
                HirExpr::Unary(op, Box::new(self.lower_expr(*inner)), span)
            }
            Expr::Binary(BinaryOp::Pipeline, left, right, span) => {
                self.lower_pipeline(*left, *right, span)
            }
            Expr::Binary(BinaryOp::OrElse, left, right, span) => HirExpr::OrElse(
                Box::new(self.lower_expr(*left)),
                Box::new(self.lower_expr(*right)),
                span,
            ),
            Expr::Binary(op, left, right, span) => HirExpr::Binary(
                op,
                Box::new(self.lower_expr(*left)),
                Box::new(self.lower_expr(*right)),
                span,
            ),
            Expr::Call(call) => self.lower_call(call),
            Expr::Dot(target, member, span) => {
                HirExpr::Dot(Box::new(self.lower_expr(*target)), member.name, span)
            }
            Expr::QuestionDot(target, member, span) => {
                HirExpr::QuestionDot(Box::new(self.lower_expr(*target)), member.name, span)
            }
            Expr::Index(target, index, span) => HirExpr::Index(
                Box::new(self.lower_expr(*target)),
                Box::new(self.lower_expr(*index)),
                span,
            ),
            Expr::List(items, span) => HirExpr::List(
                items.into_iter().map(|i| self.lower_expr(i)).collect(),
                span,
            ),
            Expr::Dict(pairs, span) => HirExpr::Dict(
                pairs
                    .into_iter()
                    .map(|(k, v)| (self.lower_expr(k), self.lower_expr(v)))
                    .collect(),
                span,
            ),
            Expr::Construct(c) => {
                let fields = c
                    .fields
                    .into_iter()
                    .map(|f| (f.name.map(|n| n.name), self.lower_expr(f.value)))
                    .collect();
                HirExpr::Construct(c.target.name, fields, c.span)
            }
            Expr::Fn(f) => {
                let params = f.params.into_iter().map(|p| self.lower_param(p)).collect();
                let mut body = Vec::new();
                for stmt in f.body {
                    self.lower_stmt(stmt, &mut body);
                }
                HirExpr::Fn(HirFunctionExpr {
                    params,
                    return_type: f.return_type,
                    body,
                    span: f.span,
                })
            }
            Expr::If(cond, then_b, else_b, span) => HirExpr::If(
                Box::new(self.lower_expr(*cond)),
                Box::new(self.lower_expr(*then_b)),
                Box::new(self.lower_expr(*else_b)),
                span,
            ),
        }
    }

    /// Desugars `left |> right` into a direct call passing `left` as the first argument.
    fn lower_pipeline(&mut self, left: Expr, right: Expr, span: SourceSpan) -> HirExpr {
        let lowered_left = self.lower_expr(left);
        let first_arg = HirCallArg {
            name: None,
            span: lowered_left.span(),
            value: lowered_left,
        };

        match right {
            Expr::Call(mut call) => {
                let callee = self.lower_expr(*call.callee);
                let mut args = Vec::with_capacity(call.args.len() + 2);
                args.push(first_arg);
                for a in call.args {
                    args.push(HirCallArg {
                        name: a.name.map(|n| n.name),
                        span: a.span,
                        value: self.lower_expr(a.value),
                    });
                }
                if let Some(block) = call.trailing_block.take() {
                    let fn_expr = self.lower_trailing_block_to_fn(block);
                    let fn_span = fn_expr.span();
                    args.push(HirCallArg {
                        name: None,
                        span: fn_span,
                        value: fn_expr,
                    });
                }
                HirExpr::Call(Box::new(callee), args, span)
            }
            _ => {
                let callee = self.lower_expr(right);
                HirExpr::Call(Box::new(callee), vec![first_arg], span)
            }
        }
    }

    /// Desugars a call expression, appending any trailing block as the final closure argument.
    fn lower_call(&mut self, call: CallExpr) -> HirExpr {
        let callee = self.lower_expr(*call.callee);
        let mut args: Vec<HirCallArg> = call
            .args
            .into_iter()
            .map(|a| HirCallArg {
                name: a.name.map(|n| n.name),
                span: a.span,
                value: self.lower_expr(a.value),
            })
            .collect();

        if let Some(block) = call.trailing_block {
            let fn_expr = self.lower_trailing_block_to_fn(block);
            let fn_span = fn_expr.span();
            args.push(HirCallArg {
                name: None,
                span: fn_span,
                value: fn_expr,
            });
        }

        HirExpr::Call(Box::new(callee), args, call.span)
    }

    fn lower_trailing_block_to_fn(&mut self, block: TrailingBlock) -> HirExpr {
        let params = block
            .params
            .into_iter()
            .map(|p| HirParam {
                name: p.name,
                is_mut: false,
                is_self: false,
                type_annotation: None,
                default: None,
                span: p.span,
            })
            .collect();
        let mut body = Vec::new();
        for stmt in block.body {
            self.lower_stmt(stmt, &mut body);
        }
        HirExpr::Fn(HirFunctionExpr {
            params,
            return_type: None,
            body,
            span: block.span,
        })
    }
}
