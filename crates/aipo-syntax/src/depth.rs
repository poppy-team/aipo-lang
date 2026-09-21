//! Iterative AST depth measurement guarding downstream recursion.
//!
//! The parser's own depth guards cap stack use *during* parsing, but some
//! shapes are built iteratively yet deeply (long `a + b + …` chains, format
//! strings with hundreds of placeholders): the resulting tree would overflow
//! the host stack inside recursive downstream walkers (HIR lowering and
//! beyond). This post-pass measures depth iteratively — with an explicit work
//! stack, so the measurer itself cannot overflow — and reports
//! `AIPO_PARSE_NESTING_TOO_DEEP` before anything recurses over the tree.
//!
//! The match arms are deliberately exhaustive: adding a recursive AST shape
//! without measuring it must be a compile error here, not a silent hole.

use aipo_ast::*;
use aipo_source::SourceSpan;

/// Maximum AST nesting (expressions and statements combined).
///
/// Rationale (see ADP-005): the 8 MiB host stack aborts past ~1000–2000 chained
/// binary nodes (~4–8 KiB per downstream level), so 2 MiB threads would abort
/// near ~250–500. The bound 128 keeps worst-case use ~1 MiB with margin, while
/// the deepest corpus nesting is single digits.
pub const MAX_AST_DEPTH: usize = 128;

enum Work<'a> {
    Stmt(&'a Stmt, usize),
    Expr(&'a Expr, usize),
}

/// Returns the span of the first node deeper than [`MAX_AST_DEPTH`], if any.
#[must_use]
pub fn over_limit_span(program: &Program) -> Option<SourceSpan> {
    let mut stack = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(decl) => {
                for stmt in &decl.body {
                    stack.push(Work::Stmt(stmt, 1));
                }
            }
            Item::Impl(block) => {
                if let Some(init) = &block.init {
                    for stmt in &init.body {
                        stack.push(Work::Stmt(stmt, 1));
                    }
                }
                for method in &block.methods {
                    for stmt in &method.body {
                        stack.push(Work::Stmt(stmt, 1));
                    }
                }
            }
            Item::Struct(_)
            | Item::Interface(_)
            | Item::Satisfy(_)
            | Item::Import(_)
            | Item::Export(_) => {}
        }
    }
    for stmt in &program.statements {
        stack.push(Work::Stmt(stmt, 1));
    }

    while let Some(work) = stack.pop() {
        match work {
            Work::Stmt(stmt, depth) => {
                if depth > MAX_AST_DEPTH {
                    return Some(stmt_span(stmt));
                }
                push_stmt_children(&mut stack, stmt, depth + 1);
            }
            Work::Expr(expr, depth) => {
                if depth > MAX_AST_DEPTH {
                    return Some(expr.span());
                }
                push_expr_children(&mut stack, expr, depth + 1);
            }
        }
    }
    None
}

fn push_stmt_children<'a>(stack: &mut Vec<Work<'a>>, stmt: &'a Stmt, depth: usize) {
    match stmt {
        Stmt::Let(_, expr, _) | Stmt::Var(_, expr, _) | Stmt::Fail(expr, _) | Stmt::Expr(expr) => {
            stack.push(Work::Expr(expr, depth))
        }
        Stmt::Assign(left, right, _) | Stmt::CompoundAssign(_, left, right, _) => {
            stack.push(Work::Expr(left, depth));
            stack.push(Work::Expr(right, depth));
        }
        Stmt::If(if_stmt) => {
            stack.push(Work::Expr(&if_stmt.condition, depth));
            push_stmts(stack, &if_stmt.then_branch, depth);
            for (cond, body) in &if_stmt.elif_branches {
                stack.push(Work::Expr(cond, depth));
                push_stmts(stack, body, depth);
            }
            if let Some(body) = &if_stmt.else_branch {
                push_stmts(stack, body, depth);
            }
        }
        Stmt::Match(match_stmt) => {
            stack.push(Work::Expr(&match_stmt.target, depth));
            for (patterns, body) in &match_stmt.when_arms {
                for pattern in patterns {
                    stack.push(Work::Expr(pattern, depth));
                }
                push_stmts(stack, body, depth);
            }
            if let Some(body) = &match_stmt.else_arm {
                push_stmts(stack, body, depth);
            }
        }
        Stmt::Loop(body, _) => push_stmts(stack, body, depth),
        Stmt::While(cond, body, _) => {
            stack.push(Work::Expr(cond, depth));
            push_stmts(stack, body, depth);
        }
        Stmt::Repeat(count, _, body, _) => {
            stack.push(Work::Expr(count, depth));
            push_stmts(stack, body, depth);
        }
        Stmt::Each(_, collection, body, _) => {
            stack.push(Work::Expr(collection, depth));
            push_stmts(stack, body, depth);
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
        Stmt::Return(expr, _) => {
            if let Some(value) = expr {
                stack.push(Work::Expr(value, depth));
            }
        }
        Stmt::Attempt(attempt) => {
            push_stmts(stack, &attempt.body, depth);
            push_stmts(stack, &attempt.failed_body, depth);
        }
        Stmt::Fn(decl) => {
            for stmt in &decl.body {
                stack.push(Work::Stmt(stmt, depth));
            }
        }
        Stmt::AwaitDo(body, _) => {
            for stmt in body {
                stack.push(Work::Stmt(stmt, depth));
            }
        }
    }
}

fn push_stmts<'a>(stack: &mut Vec<Work<'a>>, body: &'a [Stmt], depth: usize) {
    for stmt in body {
        stack.push(Work::Stmt(stmt, depth));
    }
}

fn push_expr_children<'a>(stack: &mut Vec<Work<'a>>, expr: &'a Expr, depth: usize) {
    match expr {
        Expr::Literal(_, _) | Expr::Identifier(_) => {}
        Expr::Unary(_, inner, _) => stack.push(Work::Expr(inner, depth)),
        Expr::Binary(_, left, right, _) => {
            stack.push(Work::Expr(left, depth));
            stack.push(Work::Expr(right, depth));
        }
        Expr::Call(call) => {
            stack.push(Work::Expr(&call.callee, depth));
            for arg in &call.args {
                stack.push(Work::Expr(&arg.value, depth));
            }
            if let Some(block) = &call.trailing_block {
                push_stmts(stack, &block.body, depth);
            }
        }
        Expr::Dot(target, _, _) | Expr::QuestionDot(target, _, _) => {
            stack.push(Work::Expr(target, depth));
        }
        Expr::Index(target, index, _) => {
            stack.push(Work::Expr(target, depth));
            stack.push(Work::Expr(index, depth));
        }
        Expr::Construct(construct) => {
            for field in &construct.fields {
                stack.push(Work::Expr(&field.value, depth));
            }
        }
        Expr::Fn(func) => {
            for stmt in &func.body {
                stack.push(Work::Stmt(stmt, depth));
            }
        }
        Expr::List(items, _) => {
            for item in items {
                stack.push(Work::Expr(item, depth));
            }
        }
        Expr::Dict(entries, _) => {
            for (key, value) in entries {
                stack.push(Work::Expr(key, depth));
                stack.push(Work::Expr(value, depth));
            }
        }
        Expr::If(cond, then_value, else_value, _) => {
            stack.push(Work::Expr(cond, depth));
            stack.push(Work::Expr(then_value, depth));
            stack.push(Work::Expr(else_value, depth));
        }
        Expr::Await(inner, _) => {
            stack.push(Work::Expr(inner, depth));
        }
    }
}

fn stmt_span(stmt: &Stmt) -> SourceSpan {
    match stmt {
        Stmt::Let(_, _, span)
        | Stmt::Var(_, _, span)
        | Stmt::Assign(_, _, span)
        | Stmt::CompoundAssign(_, _, _, span)
        | Stmt::Loop(_, span)
        | Stmt::While(_, _, span)
        | Stmt::Repeat(_, _, _, span)
        | Stmt::Each(_, _, _, span)
        | Stmt::Break(span)
        | Stmt::Continue(span)
        | Stmt::Return(_, span)
        | Stmt::Fail(_, span)
        | Stmt::AwaitDo(_, span) => *span,
        Stmt::Expr(expr) => expr.span(),
        Stmt::If(if_stmt) => if_stmt.span,
        Stmt::Match(match_stmt) => match_stmt.span,
        Stmt::Attempt(attempt) => attempt.span,
        Stmt::Fn(decl) => decl.span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::{Source, SourceId};

    fn parse_ok(text: &str) -> Program {
        let source = Source::new(SourceId::next(), "test.aipo", text);
        let (program, diagnostics) = crate::parse(&source);
        assert!(
            diagnostics.is_empty(),
            "fixture must parse cleanly: {diagnostics:?}"
        );
        program
    }

    #[test]
    fn test_shallow_tree_is_within_bounds() {
        let program = parse_ok("let x = 1 + 2 * 3\nio.println(x)\n");
        assert_eq!(over_limit_span(&program), None);
    }

    #[test]
    fn test_deep_chain_exceeds_bounds() {
        let chain = (0..300).map(|_| "1").collect::<Vec<_>>().join("+");
        let source = Source::new(
            SourceId::next(),
            "deep.aipo",
            &format!("io.println({chain})\n"),
        );
        let (program, _) = crate::parse(&source);
        assert!(
            over_limit_span(&program).is_some(),
            "300-deep chain must trip the bound"
        );
    }

    #[test]
    fn test_deep_tree_reports_nesting_code() {
        let chain = (0..300).map(|_| "1").collect::<Vec<_>>().join("+");
        let source = Source::new(
            SourceId::next(),
            "deep.aipo",
            &format!("io.println({chain})\n"),
        );
        let (_, diagnostics) = crate::parse(&source);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == aipo_diagnostics::DiagnosticCode::AIPO_PARSE_NESTING_TOO_DEEP),
            "deep tree reports the nesting bound: {diagnostics:?}"
        );
    }
}
