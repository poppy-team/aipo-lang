//! Span shifting for sub-parsed slices.
//!
//! Format-string placeholders are lexed and parsed from an unpadded slice of
//! the original text (padding every slice to its absolute offset made
//! placeholder-heavy files quadratic). Everything a sub-parse produces —
//! expression spans, diagnostic spans, suggestion offsets — is shifted back by
//! the slice start here, so observable spans are identical to the padded
//! implementation they replace.
//!
//! The matches are deliberately exhaustive: a new AST shape without a shift
//! arm is a compile error, not a silently wrong span.

use aipo_ast::*;
use aipo_source::SourceSpan;

fn shifted(span: SourceSpan, delta: usize) -> SourceSpan {
    SourceSpan::new(span.start + delta, span.end + delta)
}

fn shift_ident(ident: &Ident, delta: usize) -> Ident {
    Ident::new(ident.name.clone(), shifted(ident.span, delta))
}

fn shift_annotation(annotation: &TypeAnnotation, delta: usize) -> TypeAnnotation {
    TypeAnnotation {
        name: annotation.name.clone(),
        is_nullable: annotation.is_nullable,
        span: shifted(annotation.span, delta),
    }
}

fn shift_pattern(pattern: &BindingPattern, delta: usize) -> BindingPattern {
    match pattern {
        BindingPattern::Ident(ident) => BindingPattern::Ident(shift_ident(ident, delta)),
        BindingPattern::Discard(span) => BindingPattern::Discard(shifted(*span, delta)),
        BindingPattern::List(patterns, span) => BindingPattern::List(
            patterns.iter().map(|p| shift_pattern(p, delta)).collect(),
            shifted(*span, delta),
        ),
        BindingPattern::Struct(idents, span) => BindingPattern::Struct(
            idents.iter().map(|i| shift_ident(i, delta)).collect(),
            shifted(*span, delta),
        ),
    }
}

fn shift_param(param: &Param, delta: usize) -> Param {
    Param {
        name: shift_ident(&param.name, delta),
        is_mutable: param.is_mutable,
        type_annotation: param
            .type_annotation
            .as_ref()
            .map(|annotation| shift_annotation(annotation, delta)),
        default: param.default.as_ref().map(|expr| shift_expr(expr, delta)),
        span: shifted(param.span, delta),
    }
}

/// Returns `expr` with every span moved forward by `delta` bytes.
#[must_use]
pub fn shift_expr(expr: &Expr, delta: usize) -> Expr {
    match expr {
        Expr::Literal(literal, span) => Expr::Literal(literal.clone(), shifted(*span, delta)),
        Expr::Identifier(ident) => Expr::Identifier(shift_ident(ident, delta)),
        Expr::Unary(op, inner, span) => Expr::Unary(
            *op,
            Box::new(shift_expr(inner, delta)),
            shifted(*span, delta),
        ),
        Expr::Binary(op, left, right, span) => Expr::Binary(
            *op,
            Box::new(shift_expr(left, delta)),
            Box::new(shift_expr(right, delta)),
            shifted(*span, delta),
        ),
        Expr::Call(call) => Expr::Call(CallExpr {
            callee: Box::new(shift_expr(&call.callee, delta)),
            args: call
                .args
                .iter()
                .map(|arg| CallArg {
                    name: arg.name.as_ref().map(|name| shift_ident(name, delta)),
                    value: shift_expr(&arg.value, delta),
                    span: shifted(arg.span, delta),
                })
                .collect(),
            trailing_block: call.trailing_block.as_ref().map(|block| TrailingBlock {
                params: block.params.iter().map(|p| shift_ident(p, delta)).collect(),
                body: block
                    .body
                    .iter()
                    .map(|stmt| shift_stmt(stmt, delta))
                    .collect(),
                span: shifted(block.span, delta),
            }),
            span: shifted(call.span, delta),
        }),
        Expr::Dot(target, field, span) => Expr::Dot(
            Box::new(shift_expr(target, delta)),
            shift_ident(field, delta),
            shifted(*span, delta),
        ),
        Expr::QuestionDot(target, field, span) => Expr::QuestionDot(
            Box::new(shift_expr(target, delta)),
            shift_ident(field, delta),
            shifted(*span, delta),
        ),
        Expr::Index(target, index, span) => Expr::Index(
            Box::new(shift_expr(target, delta)),
            Box::new(shift_expr(index, delta)),
            shifted(*span, delta),
        ),
        Expr::Construct(construct) => Expr::Construct(ConstructExpr {
            target: shift_ident(&construct.target, delta),
            fields: construct
                .fields
                .iter()
                .map(|field| ConstructField {
                    name: field.name.as_ref().map(|name| shift_ident(name, delta)),
                    value: shift_expr(&field.value, delta),
                    span: shifted(field.span, delta),
                })
                .collect(),
            span: shifted(construct.span, delta),
        }),
        Expr::Fn(func) => Expr::Fn(FunctionExpr {
            is_async: func.is_async,
            params: func.params.iter().map(|p| shift_param(p, delta)).collect(),
            return_type: func
                .return_type
                .as_ref()
                .map(|annotation| shift_annotation(annotation, delta)),
            body: func
                .body
                .iter()
                .map(|stmt| shift_stmt(stmt, delta))
                .collect(),
            span: shifted(func.span, delta),
        }),
        Expr::List(items, span) => Expr::List(
            items.iter().map(|item| shift_expr(item, delta)).collect(),
            shifted(*span, delta),
        ),
        Expr::Dict(entries, span) => Expr::Dict(
            entries
                .iter()
                .map(|(key, value)| (shift_expr(key, delta), shift_expr(value, delta)))
                .collect(),
            shifted(*span, delta),
        ),
        Expr::If(cond, then_value, else_value, span) => Expr::If(
            Box::new(shift_expr(cond, delta)),
            Box::new(shift_expr(then_value, delta)),
            Box::new(shift_expr(else_value, delta)),
            shifted(*span, delta),
        ),
        Expr::Await(inner, span) => {
            Expr::Await(Box::new(shift_expr(inner, delta)), shifted(*span, delta))
        }
    }
}

fn shift_stmts(body: &[Stmt], delta: usize) -> Vec<Stmt> {
    body.iter().map(|stmt| shift_stmt(stmt, delta)).collect()
}

fn shift_stmt(stmt: &Stmt, delta: usize) -> Stmt {
    match stmt {
        Stmt::Let(pattern, expr, span) => Stmt::Let(
            shift_pattern(pattern, delta),
            shift_expr(expr, delta),
            shifted(*span, delta),
        ),
        Stmt::Var(pattern, expr, span) => Stmt::Var(
            shift_pattern(pattern, delta),
            shift_expr(expr, delta),
            shifted(*span, delta),
        ),
        Stmt::Assign(left, right, span) => Stmt::Assign(
            shift_expr(left, delta),
            shift_expr(right, delta),
            shifted(*span, delta),
        ),
        Stmt::CompoundAssign(op, left, right, span) => Stmt::CompoundAssign(
            *op,
            shift_expr(left, delta),
            shift_expr(right, delta),
            shifted(*span, delta),
        ),
        Stmt::If(if_stmt) => Stmt::If(IfStmt {
            condition: shift_expr(&if_stmt.condition, delta),
            then_branch: shift_stmts(&if_stmt.then_branch, delta),
            elif_branches: if_stmt
                .elif_branches
                .iter()
                .map(|(cond, body)| (shift_expr(cond, delta), shift_stmts(body, delta)))
                .collect(),
            else_branch: if_stmt
                .else_branch
                .as_ref()
                .map(|body| shift_stmts(body, delta)),
            span: shifted(if_stmt.span, delta),
        }),
        Stmt::Match(match_stmt) => Stmt::Match(MatchStmt {
            target: shift_expr(&match_stmt.target, delta),
            when_arms: match_stmt
                .when_arms
                .iter()
                .map(|(patterns, body)| {
                    (
                        patterns.iter().map(|p| shift_expr(p, delta)).collect(),
                        shift_stmts(body, delta),
                    )
                })
                .collect(),
            else_arm: match_stmt
                .else_arm
                .as_ref()
                .map(|body| shift_stmts(body, delta)),
            span: shifted(match_stmt.span, delta),
        }),
        Stmt::Loop(body, span) => Stmt::Loop(shift_stmts(body, delta), shifted(*span, delta)),
        Stmt::While(cond, body, span) => Stmt::While(
            shift_expr(cond, delta),
            shift_stmts(body, delta),
            shifted(*span, delta),
        ),
        Stmt::Repeat(count, index, body, span) => Stmt::Repeat(
            shift_expr(count, delta),
            index.as_ref().map(|name| shift_ident(name, delta)),
            shift_stmts(body, delta),
            shifted(*span, delta),
        ),
        Stmt::Each(names, collection, body, span) => Stmt::Each(
            names.iter().map(|name| shift_ident(name, delta)).collect(),
            shift_expr(collection, delta),
            shift_stmts(body, delta),
            shifted(*span, delta),
        ),
        Stmt::Break(span) => Stmt::Break(shifted(*span, delta)),
        Stmt::Continue(span) => Stmt::Continue(shifted(*span, delta)),
        Stmt::Return(expr, span) => Stmt::Return(
            expr.as_ref().map(|value| shift_expr(value, delta)),
            shifted(*span, delta),
        ),
        Stmt::Fail(expr, span) => Stmt::Fail(shift_expr(expr, delta), shifted(*span, delta)),
        Stmt::Attempt(attempt) => Stmt::Attempt(AttemptStmt {
            body: shift_stmts(&attempt.body, delta),
            err_binding: attempt
                .err_binding
                .as_ref()
                .map(|name| shift_ident(name, delta)),
            failed_body: shift_stmts(&attempt.failed_body, delta),
            span: shifted(attempt.span, delta),
        }),
        Stmt::Fn(decl) => Stmt::Fn(FunctionDecl {
            name: shift_ident(&decl.name, delta),
            is_async: decl.is_async,
            params: decl.params.iter().map(|p| shift_param(p, delta)).collect(),
            return_type: decl
                .return_type
                .as_ref()
                .map(|annotation| shift_annotation(annotation, delta)),
            body: shift_stmts(&decl.body, delta),
            span: shifted(decl.span, delta),
        }),
        Stmt::AwaitDo(body, span) => Stmt::AwaitDo(
            body.iter().map(|stmt| shift_stmt(stmt, delta)).collect(),
            shifted(*span, delta),
        ),
        Stmt::Expr(expr) => Stmt::Expr(shift_expr(expr, delta)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::{Source, SourceId};

    fn collect_idents(expr: &Expr, out: &mut Vec<(String, SourceSpan)>) {
        match expr {
            Expr::Identifier(ident) => out.push((ident.name.clone(), ident.span)),
            Expr::Unary(_, inner, _) | Expr::Dot(inner, _, _) | Expr::QuestionDot(inner, _, _) => {
                collect_idents(inner, out)
            }
            Expr::Binary(_, left, right, _) | Expr::Index(left, right, _) => {
                collect_idents(left, out);
                collect_idents(right, out);
            }
            Expr::Call(call) => {
                collect_idents(&call.callee, out);
                for arg in &call.args {
                    collect_idents(&arg.value, out);
                }
            }
            Expr::If(a, b, c, _) => {
                collect_idents(a, out);
                collect_idents(b, out);
                collect_idents(c, out);
            }
            Expr::List(items, _) => {
                for item in items {
                    collect_idents(item, out);
                }
            }
            _ => {}
        }
    }

    fn parse(text: &str) -> (Source, aipo_ast::Program) {
        let source = Source::new(SourceId::next(), "shift.aipo", text);
        let (program, diagnostics) = crate::parse(&source);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        (source, program)
    }

    #[test]
    fn test_placeholder_spans_point_at_source_text() {
        // Padding before the f-string moves every absolute offset: shifted
        // spans must still slice to the exact placeholder text.
        let padding = "# pad\n".repeat(50);
        let text = format!("{padding}io.println(f\"total: {{n}}!\")\n");
        let (source, program) = parse(&text);
        let mut idents = Vec::new();
        for stmt in program
            .statements
            .iter()
            .chain(program.items.iter().flat_map(|_| [].iter()))
        {
            if let aipo_ast::Stmt::Expr(expr) = stmt {
                collect_idents(expr, &mut idents);
            }
        }
        let names: Vec<String> = idents.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"n".to_string()), "{names:?}");
        assert!(names.contains(&"String".to_string()), "{names:?}");
        for (name, span) in idents {
            if name == "String" {
                // Synthetic conversion callee: its span is the whole
                // placeholder by construction (unchanged behavior).
                assert_eq!(source.slice(span), Some("{n}"));
            } else {
                assert_eq!(
                    source.slice(span),
                    Some(name.as_str()),
                    "identifier span slices to its text"
                );
            }
        }
    }
}

#[cfg(test)]
mod exotic_tests {
    use super::shift_expr;
    use aipo_source::{Source, SourceId};

    #[test]
    fn test_exotic_placeholder_spans_stay_inside_source() {
        let padding = "# filler line\n".repeat(30);
        let text = format!("{padding}io.println(f\"{{[1, 2].transform(fn (x) return x end)}}\")\n");
        let source = Source::new(SourceId::next(), "exotic.aipo", &text);
        let (program, diagnostics) = crate::parse(&source);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let shifted = shift_expr(
            &aipo_ast::Expr::Identifier(aipo_ast::Ident::new(
                "probe".to_string(),
                aipo_source::SourceSpan::empty(0),
            )),
            10,
        );
        let _ = shifted;
        // Every identifier in the parsed program slices inside the file.
        fn walk(expr: &aipo_ast::Expr, source: &Source) {
            match expr {
                aipo_ast::Expr::Identifier(ident) => {
                    assert!(source.validate_span(ident.span).is_ok());
                }
                aipo_ast::Expr::Unary(_, inner, _)
                | aipo_ast::Expr::Dot(inner, _, _)
                | aipo_ast::Expr::QuestionDot(inner, _, _) => walk(inner, source),
                aipo_ast::Expr::Binary(_, left, right, _)
                | aipo_ast::Expr::Index(left, right, _) => {
                    walk(left, source);
                    walk(right, source);
                }
                aipo_ast::Expr::Call(call) => {
                    walk(&call.callee, source);
                    for arg in &call.args {
                        walk(&arg.value, source);
                    }
                }
                aipo_ast::Expr::If(a, b, c, _) => {
                    walk(a, source);
                    walk(b, source);
                    walk(c, source);
                }
                aipo_ast::Expr::List(items, _) => {
                    for item in items {
                        walk(item, source);
                    }
                }
                _ => {}
            }
        }
        for stmt in &program.statements {
            if let aipo_ast::Stmt::Expr(expr) = stmt {
                walk(expr, &source);
            }
        }
    }
}
