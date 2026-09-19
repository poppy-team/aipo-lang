//! Recursive descent parser with Pratt expression parsing and error recovery.

use aipo_ast::*;
use aipo_diagnostics::{Diagnostic, DiagnosticCode};

/// Largest integer the runtime treats as safe (`2^53 - 1`).
///
/// Used as the implicit end bound of an open-ended slice, where the runtime clamps every
/// slice bound into `0..=len` and any value at or above `len` therefore means "to the end".
const SAFE_MAX_INT: i64 = 9_007_199_254_740_991;
use aipo_lexer::{Lexer, StringPrefix as LexerStringPrefix, Token, TokenKind};
use aipo_source::{Source, SourceId, SourceSpan};

/// Finds the `}` that closes the placeholder body starting at `body_start`.
///
/// Braces may nest inside a placeholder (a dict literal or a nested format string), so
/// the scan is depth aware.
fn find_placeholder_end(bytes: &[u8], body_start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = body_start;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// Resolves the `{{`/`}}` literal-brace escapes of a format-string segment.
fn unescape_braces(text: &str) -> String {
    text.replace("{{", "{").replace("}}", "}")
}

/// Parser state maintaining token stream, cursor position, and diagnostic collection.
pub struct Parser<'a> {
    source: &'a Source,
    tokens: Vec<Token>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    /// Constructs a new parser from source and token list.
    #[must_use]
    pub fn new(source: &'a Source, tokens: Vec<Token>) -> Self {
        // Filter out non-significant comments; keep significant newlines
        let tokens: Vec<Token> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
            .collect();

        Self {
            source,
            tokens,
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Parses a single expression from the token stream.
    ///
    /// Exposed for format-string interpolation, which parses the expression embedded in
    /// a `{...}` placeholder out of the same source buffer.
    pub fn parse_expression(&mut self) -> Option<Expr> {
        self.parse_expr()
    }

    /// Desugars a format string into a concatenation of text segments and `String(...)`.
    ///
    /// `f"total: {n}!"` becomes `"total: " + String(n) + "!"` so interpolation and the
    /// explicit `String(value)` conversion share one definition of textual rendering
    /// (`Aipo V1 — Language Reference`: interpolation is the preferred composition form).
    ///
    /// Both the text segments and the placeholder expressions are re-lexed from the
    /// original buffer, so literal escapes are processed by the single lexer
    /// implementation and every produced span points at a real file position.
    /// `{{` and `}}` are the literal-brace escapes.
    fn parse_format_string(&mut self, span: SourceSpan, prefix: StringPrefix) -> Option<Expr> {
        let text = self.source.text();
        let raw = &text[span.start..span.end];
        let prefix_len = raw
            .chars()
            .take_while(|ch| *ch == 'f' || *ch == 'r')
            .count();
        let quote_len = if raw[prefix_len..].starts_with("\"\"\"") {
            3
        } else {
            1
        };

        let inner_start = span.start + prefix_len + quote_len;
        let inner_end = span.end.saturating_sub(quote_len);
        if inner_end < inner_start {
            return None;
        }

        let inner = &text[inner_start..inner_end];
        let is_raw = matches!(prefix, StringPrefix::FormatRaw);
        let bytes = inner.as_bytes();
        let mut parts: Vec<Expr> = Vec::new();
        let mut segment_start = 0usize;
        let mut index = 0usize;

        while index < bytes.len() {
            match bytes[index] {
                b'{' if bytes.get(index + 1) == Some(&b'{') => index += 2,
                b'}' if bytes.get(index + 1) == Some(&b'}') => index += 2,
                b'{' => {
                    self.push_format_segment(segment_start, index, inner_start, is_raw, &mut parts);

                    let body_start = index + 1;
                    let Some(body_end) = find_placeholder_end(bytes, body_start) else {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                                "unclosed interpolation placeholder in format string",
                            )
                            .with_primary_span(self.source, span),
                        );
                        return None;
                    };

                    let expression = self
                        .parse_slice_expression(inner_start + body_start, inner_start + body_end)?;
                    let call_span =
                        SourceSpan::new(inner_start + index, inner_start + body_end + 1);
                    let callee = Expr::Identifier(Ident::new("String".to_string(), call_span));
                    let argument = CallArg {
                        name: None,
                        value: expression,
                        span: call_span,
                    };
                    parts.push(Expr::Call(CallExpr {
                        callee: Box::new(callee),
                        args: vec![argument],
                        trailing_block: None,
                        span: call_span,
                    }));

                    index = body_end + 1;
                    segment_start = index;
                }
                b'}' => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                            "unmatched '}' in format string: use '}}' for a literal brace",
                        )
                        .with_primary_span(self.source, span),
                    );
                    return None;
                }
                _ => index += 1,
            }
        }

        self.push_format_segment(segment_start, bytes.len(), inner_start, is_raw, &mut parts);

        let mut parts = parts.into_iter();
        let Some(first) = parts.next() else {
            return Some(Expr::Literal(
                Literal::String(String::new(), StringPrefix::Normal),
                span,
            ));
        };

        let folded = parts.fold(first, |left, right| {
            Expr::Binary(BinaryOp::Add, Box::new(left), Box::new(right), span)
        });
        Some(folded)
    }

    /// Appends the format-string text between two offsets as a `String` literal.
    fn push_format_segment(
        &mut self,
        from: usize,
        to: usize,
        inner_start: usize,
        is_raw: bool,
        parts: &mut Vec<Expr>,
    ) {
        if from >= to {
            return;
        }

        let segment_span = SourceSpan::new(inner_start + from, inner_start + to);
        let Some(content) = self.lex_slice_string(inner_start + from, inner_start + to, is_raw)
        else {
            return;
        };
        if content.is_empty() {
            return;
        }

        parts.push(Expr::Literal(
            Literal::String(unescape_braces(&content), StringPrefix::Normal),
            segment_span,
        ));
    }

    /// Re-lexes a slice of the original text as a string literal and returns its content.
    fn lex_slice_string(&self, start: usize, end: usize, is_raw: bool) -> Option<String> {
        let mut padded = " ".repeat(start);
        padded.push_str(if is_raw { "r\"" } else { "\"" });
        padded.push_str(&self.source.text()[start..end]);
        padded.push('"');

        let sub_source = Source::new(SourceId::next(), self.source.name(), &padded);
        let (tokens, diagnostics) = Lexer::new(&sub_source).tokenize();
        if !diagnostics.is_empty() {
            return None;
        }

        match tokens.first().map(|token| &token.kind) {
            Some(TokenKind::StringLiteral { content, .. }) => Some(content.clone()),
            _ => None,
        }
    }

    /// Parses a slice of the original text as a single expression.
    ///
    /// The sub-source pads everything before `start` with spaces, so the sub-parse sees
    /// the real byte offsets and every span it produces belongs to the original file.
    fn parse_slice_expression(&mut self, start: usize, end: usize) -> Option<Expr> {
        if start > end || end > self.source.text().len() {
            return None;
        }

        let mut padded = " ".repeat(start);
        padded.push_str(&self.source.text()[start..end]);
        padded.push('\n');

        let sub_source = Source::new(SourceId::next(), self.source.name(), &padded);
        let (tokens, diagnostics) = Lexer::new(&sub_source).tokenize();
        if let Some(first) = diagnostics.first() {
            self.diagnostics.push(first.clone());
            return None;
        }

        let mut sub = Parser::new(&sub_source, tokens);
        let expression = sub.parse_expression();
        self.diagnostics.append(&mut sub.diagnostics);

        if expression.is_none() {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                    "format string placeholder must contain a single expression",
                )
                .with_primary_span(self.source, SourceSpan::new(start, end)),
            );
        }
        expression
    }

    /// Parses the entire token stream into a `Program`.
    pub fn parse(mut self) -> (Program, Vec<Diagnostic>) {
        let mut items = Vec::new();
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            if self.is_item_start() {
                if let Some(item) = self.parse_item() {
                    items.push(item);
                }
            } else if let Some(stmt) = self.parse_stmt() {
                statements.push(stmt);
            } else {
                self.synchronize();
            }
            self.skip_newlines();
        }

        let span = if let (Some(first), Some(last)) = (self.tokens.first(), self.tokens.last()) {
            first.span.merge(last.span)
        } else {
            SourceSpan::empty(0)
        };

        (
            Program {
                items,
                statements,
                span,
            },
            self.diagnostics,
        )
    }

    // --- Helpers ---

    fn peek(&self) -> &TokenKind {
        self.tokens
            .get(self.cursor)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn peek_ahead(&self, offset: usize) -> &TokenKind {
        self.tokens
            .get(self.cursor + offset)
            .map(|t| &t.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.tokens
                .last()
                .cloned()
                .unwrap_or_else(|| Token::new(TokenKind::Eof, SourceSpan::empty(self.source.len())))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn check(&self, expected: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(expected)
        }
    }

    fn match_token(&mut self, expected: &TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), TokenKind::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, expected: &TokenKind, msg: &str) -> Option<Token> {
        if self.check(expected) {
            Some(self.advance())
        } else {
            let span = self
                .peek_token()
                .map(|t| t.span)
                .unwrap_or_else(|| SourceSpan::empty(self.source.len()));
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN, msg)
                    .with_primary_span(self.source, span),
            );
            None
        }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(
                self.peek(),
                TokenKind::Let
                    | TokenKind::Var
                    | TokenKind::Fn
                    | TokenKind::Struct
                    | TokenKind::Impl
                    | TokenKind::Interface
                    | TokenKind::Satisfy
                    | TokenKind::Import
                    | TokenKind::Export
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Loop
                    | TokenKind::Repeat
                    | TokenKind::Each
                    | TokenKind::Return
                    | TokenKind::End
            ) {
                return;
            }
            let tok = self.advance();
            if matches!(tok.kind, TokenKind::Newline) {
                return;
            }
        }
    }

    fn is_item_start(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Impl
                | TokenKind::Interface
                | TokenKind::Satisfy
                | TokenKind::Import
                | TokenKind::Export
        )
    }

    // --- Items ---

    fn parse_item(&mut self) -> Option<Item> {
        match self.peek() {
            TokenKind::Fn => self.parse_function_decl().map(Item::Fn),
            TokenKind::Struct => self.parse_struct_decl().map(Item::Struct),
            TokenKind::Impl => self.parse_impl_block().map(Item::Impl),
            TokenKind::Interface => self.parse_interface_decl().map(Item::Interface),
            TokenKind::Satisfy => self.parse_satisfy_decl().map(Item::Satisfy),
            TokenKind::Import => self.parse_import_decl().map(Item::Import),
            TokenKind::Export => self.parse_export_decl().map(Item::Export),
            _ => None,
        }
    }

    fn parse_struct_decl(&mut self) -> Option<StructDecl> {
        let start = self.advance().span; // 'struct'
        let name = self.parse_ident()?;
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            let is_fixed = self.match_token(&TokenKind::Fixed);
            let _ = self.match_token(&TokenKind::Var);
            let field_name = self.parse_ident()?;
            let default = if self.match_token(&TokenKind::Equal) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            let span = if let Some(def) = &default {
                field_name.span.merge(def.span())
            } else {
                field_name.span
            };
            fields.push(StructField {
                name: field_name,
                is_fixed,
                default,
                span,
            });
            self.skip_newlines();
        }

        let end_span = self
            .expect(
                &TokenKind::End,
                "expected 'end' to close struct declaration",
            )?
            .span;
        Some(StructDecl {
            name,
            fields,
            span: start.merge(end_span),
        })
    }

    fn parse_impl_block(&mut self) -> Option<ImplBlock> {
        let start = self.advance().span; // 'impl'
        let target = self.parse_ident()?;
        self.skip_newlines();

        let mut init = None;
        let mut invariant = None;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if self.check(&TokenKind::Init) {
                init = self.parse_init_hook();
            } else if self.check(&TokenKind::Invariant) {
                invariant = self.parse_invariant_hook();
            } else if self.check(&TokenKind::Fn) {
                if let Some(method) = self.parse_function_decl() {
                    methods.push(method);
                }
            } else {
                self.advance();
            }
            self.skip_newlines();
        }

        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close impl block")?
            .span;
        Some(ImplBlock {
            target,
            init,
            invariant,
            methods,
            span: start.merge(end_span),
        })
    }

    fn parse_init_hook(&mut self) -> Option<FunctionDecl> {
        let start = self.advance().span; // 'init'
        self.expect(&TokenKind::LParen, "expected '(' after init")?;
        let mut params = self.parse_params()?;
        self.expect(&TokenKind::RParen, "expected ')' after init params")?;
        self.skip_newlines();

        // Canon declares `init` without an explicit receiver: `self` is implicitly the instance
        // still being constructed and is mutable during the construction phase. Injecting the
        // receiver here (only when the author did not write one) means every later stage sees
        // exactly the same shape as for an explicitly declared `self!`.
        if !params
            .first()
            .is_some_and(|param| param.name.name == "self")
        {
            params.insert(
                0,
                Param {
                    name: Ident::new("self".into(), start),
                    is_mutable: true,
                    type_annotation: None,
                    default: None,
                    span: start,
                },
            );
        }

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                body.push(stmt);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close init hook")?
            .span;

        Some(FunctionDecl {
            name: Ident::new("init".into(), start),
            params,
            return_type: None,
            body,
            span: start.merge(end_span),
        })
    }

    fn parse_invariant_hook(&mut self) -> Option<InvariantHook> {
        let start = self.advance().span; // 'invariant'
        self.expect(&TokenKind::LParen, "expected '(' after invariant")?;
        self.expect(&TokenKind::RParen, "expected ')' after invariant")?;
        self.skip_newlines();

        let mut conditions = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(expr) = self.parse_expr() {
                conditions.push(expr);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close invariant hook")?
            .span;

        Some(InvariantHook {
            conditions,
            span: start.merge(end_span),
        })
    }

    fn parse_interface_decl(&mut self) -> Option<InterfaceDecl> {
        let start = self.advance().span; // 'interface'
        let name = self.parse_ident()?;
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if self.check(&TokenKind::Fn) {
                if let Some(method) = self.parse_function_signature() {
                    methods.push(method);
                }
            } else {
                self.advance();
            }
            self.skip_newlines();
        }

        let end_span = self
            .expect(
                &TokenKind::End,
                "expected 'end' to close interface declaration",
            )?
            .span;
        Some(InterfaceDecl {
            name,
            methods,
            span: start.merge(end_span),
        })
    }

    fn parse_satisfy_decl(&mut self) -> Option<SatisfyDecl> {
        let start = self.advance().span; // 'satisfy'
        let target = self.parse_ident()?;
        self.expect(&TokenKind::Colon, "expected ':' after satisfy target")?;

        let mut interfaces = Vec::new();
        loop {
            interfaces.push(self.parse_ident()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        let end_span = interfaces.last().map(|i| i.span).unwrap_or(start);
        Some(SatisfyDecl {
            target,
            interfaces,
            span: start.merge(end_span),
        })
    }

    fn parse_import_decl(&mut self) -> Option<ImportDecl> {
        let start = self.advance().span; // 'import'
        let module_name = self.parse_ident()?;
        let alias = if self.match_token(&TokenKind::As) {
            Some(self.parse_ident()?)
        } else {
            None
        };

        let mut names = Vec::new();
        if self.match_token(&TokenKind::Colon) {
            loop {
                names.push(self.parse_ident()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = names
            .last()
            .map(|i| i.span)
            .or_else(|| alias.as_ref().map(|a| a.span))
            .unwrap_or(module_name.span);

        Some(ImportDecl {
            module_name,
            alias,
            names,
            span: start.merge(end_span),
        })
    }

    fn parse_export_decl(&mut self) -> Option<ExportDecl> {
        let start = self.advance().span; // 'export'
        let mut names = Vec::new();
        loop {
            names.push(self.parse_ident()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let end_span = names.last().map(|i| i.span).unwrap_or(start);
        Some(ExportDecl {
            names,
            span: start.merge(end_span),
        })
    }

    // --- Statements ---

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Var => self.parse_var_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Repeat => self.parse_repeat_stmt(),
            TokenKind::Each => self.parse_each_stmt(),
            TokenKind::Break => {
                let span = self.advance().span;
                Some(Stmt::Break(span))
            }
            TokenKind::Continue => {
                let span = self.advance().span;
                Some(Stmt::Continue(span))
            }
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Fail => self.parse_fail_stmt(),
            TokenKind::Attempt => self.parse_attempt_stmt(),
            TokenKind::Fn => {
                // A named `fn name(params) ... end` inside a statement position is a local
                // function declaration: canon makes its binding exist when execution reaches
                // the declaration, visible inside its own body for recursion. The name must
                // survive lowering, so it is kept instead of being collapsed into an
                // anonymous closure expression.
                self.parse_function_decl().map(Stmt::Fn)
            }
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'let'
        let pattern = self.parse_binding_pattern()?;
        self.expect(&TokenKind::Equal, "expected '=' in let binding")?;
        let expr = self.parse_expr()?;
        let span = start.merge(expr.span());
        Some(Stmt::Let(pattern, expr, span))
    }

    fn parse_var_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'var'
        let pattern = self.parse_binding_pattern()?;
        self.expect(&TokenKind::Equal, "expected '=' in var binding")?;
        let expr = self.parse_expr()?;
        let span = start.merge(expr.span());
        Some(Stmt::Var(pattern, expr, span))
    }

    fn parse_binding_pattern(&mut self) -> Option<BindingPattern> {
        if self.match_token(&TokenKind::Discard) {
            let span = self.tokens[self.cursor - 1].span;
            Some(BindingPattern::Discard(span))
        } else if self.match_token(&TokenKind::LBracket) {
            let start = self.tokens[self.cursor - 1].span;
            let mut list = Vec::new();
            while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
                list.push(self.parse_binding_pattern()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            let end = self
                .expect(&TokenKind::RBracket, "expected ']' after binding list")?
                .span;
            Some(BindingPattern::List(list, start.merge(end)))
        } else if self.match_token(&TokenKind::LBrace) {
            let start = self.tokens[self.cursor - 1].span;
            let mut fields = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                fields.push(self.parse_ident()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            let end = self
                .expect(&TokenKind::RBrace, "expected '}' after binding struct")?
                .span;
            Some(BindingPattern::Struct(fields, start.merge(end)))
        } else {
            let ident = self.parse_ident()?;
            Some(BindingPattern::Ident(ident))
        }
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expr()?;

        // Check for simple assignment '='
        if self.match_token(&TokenKind::Equal) {
            let value = self.parse_expr()?;
            let span = expr.span().merge(value.span());
            return Some(Stmt::Assign(expr, value, span));
        }

        // Check for compound assignments
        let compound_op = if self.match_token(&TokenKind::PlusEq) {
            Some(BinaryOp::Add)
        } else if self.match_token(&TokenKind::MinusEq) {
            Some(BinaryOp::Sub)
        } else if self.match_token(&TokenKind::StarEq) {
            Some(BinaryOp::Mul)
        } else if self.match_token(&TokenKind::SlashEq) {
            Some(BinaryOp::Div)
        } else if self.match_token(&TokenKind::DivEq) {
            Some(BinaryOp::IntDiv)
        } else if self.match_token(&TokenKind::PercentEq) {
            Some(BinaryOp::Mod)
        } else {
            None
        };

        if let Some(op) = compound_op {
            let value = self.parse_expr()?;
            let span = expr.span().merge(value.span());
            return Some(Stmt::CompoundAssign(op, expr, value, span));
        }

        Some(Stmt::Expr(expr))
    }

    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'if'
        let condition = self.parse_expr()?;

        // Check for inline if: `if c then stmt [else stmt]`
        if self.match_token(&TokenKind::Then) {
            if self.check(&TokenKind::Newline) {
                self.advance();
            } else {
                let then_stmt = self.parse_stmt()?;
                let else_branch = if self.match_token(&TokenKind::Else) {
                    Some(vec![self.parse_stmt()?])
                } else {
                    None
                };
                let end_span = else_branch
                    .as_ref()
                    .and_then(|b| b.last())
                    .map(stmt_span)
                    .unwrap_or_else(|| stmt_span(&then_stmt));

                return Some(Stmt::If(IfStmt {
                    condition,
                    then_branch: vec![then_stmt],
                    elif_branches: Vec::new(),
                    else_branch,
                    span: start.merge(end_span),
                }));
            }
        }

        self.skip_newlines();
        let mut then_branch = Vec::new();
        while !self.check(&TokenKind::Elif)
            && !self.check(&TokenKind::Else)
            && !self.check(&TokenKind::End)
            && !self.is_at_end()
        {
            if let Some(s) = self.parse_stmt() {
                then_branch.push(s);
            }
            self.skip_newlines();
        }

        let mut elif_branches = Vec::new();
        while self.match_token(&TokenKind::Elif) {
            let elif_cond = self.parse_expr()?;
            let _ = self.match_token(&TokenKind::Then);
            self.skip_newlines();
            let mut elif_body = Vec::new();
            while !self.check(&TokenKind::Elif)
                && !self.check(&TokenKind::Else)
                && !self.check(&TokenKind::End)
                && !self.is_at_end()
            {
                if let Some(s) = self.parse_stmt() {
                    elif_body.push(s);
                }
                self.skip_newlines();
            }
            elif_branches.push((elif_cond, elif_body));
        }

        let else_branch = if self.match_token(&TokenKind::Else) {
            self.skip_newlines();
            let mut else_body = Vec::new();
            while !self.check(&TokenKind::End) && !self.is_at_end() {
                if let Some(s) = self.parse_stmt() {
                    else_body.push(s);
                }
                self.skip_newlines();
            }
            Some(else_body)
        } else {
            None
        };

        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close if statement")?
            .span;

        Some(Stmt::If(IfStmt {
            condition,
            then_branch,
            elif_branches,
            else_branch,
            span: start.merge(end_span),
        }))
    }

    fn parse_match_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'match'
        let target = self.parse_expr()?;
        self.skip_newlines();

        let mut when_arms = Vec::new();
        while self.match_token(&TokenKind::When) {
            let mut patterns = Vec::new();
            loop {
                patterns.push(self.parse_expr()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            let _ = self.match_token(&TokenKind::Then);
            self.skip_newlines();

            let mut arm_body = Vec::new();
            while !self.check(&TokenKind::When)
                && !self.check(&TokenKind::Else)
                && !self.check(&TokenKind::End)
                && !self.is_at_end()
            {
                if let Some(s) = self.parse_stmt() {
                    arm_body.push(s);
                }
                self.skip_newlines();
            }
            when_arms.push((patterns, arm_body));
        }

        let else_arm = if self.match_token(&TokenKind::Else) {
            self.skip_newlines();
            let mut else_body = Vec::new();
            while !self.check(&TokenKind::End) && !self.is_at_end() {
                if let Some(s) = self.parse_stmt() {
                    else_body.push(s);
                }
                self.skip_newlines();
            }
            Some(else_body)
        } else {
            None
        };

        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close match statement")?
            .span;

        Some(Stmt::Match(MatchStmt {
            target,
            when_arms,
            else_arm,
            span: start.merge(end_span),
        }))
    }

    fn parse_loop_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'loop'
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close loop")?
            .span;
        Some(Stmt::Loop(body, start.merge(end_span)))
    }

    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'while'
        let condition = self.parse_expr()?;
        self.skip_newlines();
        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close while")?
            .span;
        Some(Stmt::While(condition, body, start.merge(end_span)))
    }

    fn parse_repeat_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'repeat'
        let count = self.parse_expr()?;
        let index_var = if self.match_token(&TokenKind::As) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close repeat")?
            .span;
        Some(Stmt::Repeat(count, index_var, body, start.merge(end_span)))
    }

    fn parse_each_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'each'
        let mut bindings = Vec::new();
        loop {
            bindings.push(self.parse_ident()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::In, "expected 'in' in each loop")?;
        let iterable = self.parse_expr()?;
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close each loop")?
            .span;
        Some(Stmt::Each(bindings, iterable, body, start.merge(end_span)))
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'return'
        let value = if matches!(
            self.peek(),
            TokenKind::Newline | TokenKind::End | TokenKind::Eof
        ) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        let span = if let Some(val) = &value {
            start.merge(val.span())
        } else {
            start
        };
        Some(Stmt::Return(value, span))
    }

    /// Parses the expression between `[` and `]`, normalizing omitted slice bounds.
    ///
    /// Canon defines slicing as half-open and explicitly allows omitted limits. The runtime
    /// already clamps both ends of a slice into `0..=len`, so an omitted bound is exactly
    /// equivalent to the corresponding extreme value and needs no new runtime kind: an
    /// omitted start means index `0`, and an omitted end means "up to the last element",
    /// spelled with the largest safe Aipo integer.
    fn parse_index_expr(&mut self) -> Option<Expr> {
        if self.check(&TokenKind::DotDot) {
            let dot = self.advance().span;
            let start = Expr::Literal(Literal::Int("0".to_string()), dot);
            let end = if self.check(&TokenKind::RBracket) {
                Expr::Literal(Literal::Int(SAFE_MAX_INT.to_string()), dot)
            } else {
                self.parse_pratt_expr(Precedence::Range)?
            };
            let span = dot.merge(end.span());
            return Some(Expr::Binary(
                BinaryOp::Range,
                Box::new(start),
                Box::new(end),
                span,
            ));
        }

        // Stop *before* `..` so the slice bound is not swallowed by the range operator.
        let start = self.parse_pratt_expr(Precedence::Range)?;
        if self.check(&TokenKind::DotDot) {
            let dot = self.advance().span;
            let end = if self.check(&TokenKind::RBracket) {
                Expr::Literal(Literal::Int(SAFE_MAX_INT.to_string()), dot)
            } else {
                self.parse_pratt_expr(Precedence::Range)?
            };
            let span = start.span().merge(end.span());
            return Some(Expr::Binary(
                BinaryOp::Range,
                Box::new(start),
                Box::new(end),
                span,
            ));
        }
        // Nothing to slice: finish any looser operators (`xs[a == b]`).
        self.continue_pratt(start, Precedence::Lowest)
    }

    fn parse_fail_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'fail'
        let value = self.parse_expr()?;
        let span = start.merge(value.span());
        Some(Stmt::Fail(value, span))
    }

    fn parse_attempt_stmt(&mut self) -> Option<Stmt> {
        let start = self.advance().span; // 'attempt'
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::Failed) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }

        self.expect(&TokenKind::Failed, "expected 'failed' in attempt statement")?;
        let err_binding = if matches!(self.peek(), TokenKind::Newline | TokenKind::End)
            || self.match_token(&TokenKind::Discard)
        {
            None
        } else {
            Some(self.parse_ident()?)
        };
        self.skip_newlines();

        let mut failed_body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                failed_body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close attempt statement")?
            .span;

        Some(Stmt::Attempt(AttemptStmt {
            body,
            err_binding,
            failed_body,
            span: start.merge(end_span),
        }))
    }

    fn parse_function_decl(&mut self) -> Option<FunctionDecl> {
        let start = self.advance().span; // 'fn'
        let name = self.parse_ident()?;
        self.expect(
            &TokenKind::LParen,
            "expected '(' in function parameter list",
        )?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;

        let return_type = self.parse_optional_return_type();
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(
                &TokenKind::End,
                "expected 'end' to close function declaration",
            )?
            .span;

        Some(FunctionDecl {
            name,
            params,
            return_type,
            body,
            span: start.merge(end_span),
        })
    }

    fn parse_function_signature(&mut self) -> Option<FunctionDecl> {
        let start = self.advance().span; // 'fn'
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LParen, "expected '(' in method signature")?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;
        let return_type = self.parse_optional_return_type();
        let end_span = return_type.as_ref().map(|r| r.span).unwrap_or(name.span);

        Some(FunctionDecl {
            name,
            params,
            return_type,
            body: Vec::new(),
            span: start.merge(end_span),
        })
    }

    fn parse_params(&mut self) -> Option<Vec<Param>> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) && !self.is_at_end() {
            let is_self_mut = self.match_token(&TokenKind::SelfMut);
            let (param_name, is_mut) = if is_self_mut {
                let span = self.tokens[self.cursor - 1].span;
                (Ident::new("self".into(), span), true)
            } else if self.match_token(&TokenKind::SelfVal) {
                let span = self.tokens[self.cursor - 1].span;
                (Ident::new("self".into(), span), false)
            } else {
                let id = self.parse_ident()?;
                let mut_marker = self.match_token(&TokenKind::Bang);
                (id, mut_marker)
            };

            let type_annotation = if self.match_token(&TokenKind::Colon) {
                let type_ident = self.parse_ident()?;
                let is_nullable = self.match_token(&TokenKind::Question);
                let span = if is_nullable {
                    type_ident.span.merge(self.tokens[self.cursor - 1].span)
                } else {
                    type_ident.span
                };
                Some(TypeAnnotation {
                    name: type_ident.name,
                    is_nullable,
                    span,
                })
            } else {
                None
            };

            let default = if self.match_token(&TokenKind::Equal) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            let span = param_name.span;
            params.push(Param {
                name: param_name,
                is_mutable: is_mut,
                type_annotation,
                default,
                span,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        Some(params)
    }

    fn parse_optional_return_type(&mut self) -> Option<TypeAnnotation> {
        // Look for `-> Type[?]`
        if self.match_token(&TokenKind::Minus) && self.match_token(&TokenKind::Greater) {
            let type_ident = self.parse_ident()?;
            let is_nullable = self.match_token(&TokenKind::Question);
            let span = if is_nullable {
                type_ident.span.merge(self.tokens[self.cursor - 1].span)
            } else {
                type_ident.span
            };
            Some(TypeAnnotation {
                name: type_ident.name,
                is_nullable,
                span,
            })
        } else {
            None
        }
    }

    // --- Pratt Expression Parsing ---

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_pratt_expr(Precedence::Lowest)
    }

    fn parse_pratt_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        let left = self.parse_prefix_expr()?;
        self.continue_pratt(left, precedence)
    }

    /// Keeps consuming infix operators above `precedence`, continuing from `left`.
    fn continue_pratt(&mut self, mut left: Expr, precedence: Precedence) -> Option<Expr> {
        while !self.is_at_end() && precedence < self.peek_precedence() {
            left = self.parse_infix_expr(left)?;
        }

        Some(left)
    }

    fn parse_prefix_expr(&mut self) -> Option<Expr> {
        match self.peek().clone() {
            TokenKind::IntLiteral(digits) => {
                let span = self.advance().span;
                Some(Expr::Literal(Literal::Int(digits), span))
            }
            TokenKind::FloatLiteral(digits) => {
                let span = self.advance().span;
                Some(Expr::Literal(Literal::Float(digits), span))
            }
            TokenKind::True => {
                let span = self.advance().span;
                Some(Expr::Literal(Literal::Bool(true), span))
            }
            TokenKind::False => {
                let span = self.advance().span;
                Some(Expr::Literal(Literal::Bool(false), span))
            }
            TokenKind::None => {
                let span = self.advance().span;
                Some(Expr::Literal(Literal::None, span))
            }
            TokenKind::StringLiteral {
                content,
                prefix,
                is_multiline: _,
            } => {
                let span = self.advance().span;
                let ast_prefix = match prefix {
                    LexerStringPrefix::Normal => StringPrefix::Normal,
                    LexerStringPrefix::Format => StringPrefix::Format,
                    LexerStringPrefix::Raw => StringPrefix::Raw,
                    LexerStringPrefix::FormatRaw => StringPrefix::FormatRaw,
                };

                // Format strings are desugared: `f"a{x}b"` becomes `"a" + String(x) + "b"`,
                // so interpolation and the explicit `String(value)` conversion share one
                // definition of textual rendering.
                if matches!(ast_prefix, StringPrefix::Format | StringPrefix::FormatRaw) {
                    return self.parse_format_string(span, ast_prefix);
                }

                Some(Expr::Literal(Literal::String(content, ast_prefix), span))
            }
            TokenKind::SelfVal => {
                let span = self.advance().span;
                Some(Expr::Identifier(Ident::new("self".into(), span)))
            }
            TokenKind::Identifier(name) => {
                let span = self.advance().span;
                let ident = Ident::new(name, span);

                // Check for struct construction `Type{...}`
                if self.check(&TokenKind::LBrace)
                    && ident.name.chars().next().is_some_and(|c| c.is_uppercase())
                {
                    return self.parse_construct_expr(ident);
                }

                Some(Expr::Identifier(ident))
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(
                    &TokenKind::RParen,
                    "expected ')' after parenthesized expression",
                )?;
                Some(inner)
            }
            TokenKind::LBracket => self.parse_list_expr(),
            TokenKind::LBrace => self.parse_dict_expr(),
            TokenKind::Minus => {
                let span = self.advance().span;
                let operand = self.parse_pratt_expr(Precedence::Prefix)?;
                let full_span = span.merge(operand.span());
                Some(Expr::Unary(UnaryOp::Neg, Box::new(operand), full_span))
            }
            TokenKind::Plus => {
                let span = self.advance().span;
                let operand = self.parse_pratt_expr(Precedence::Prefix)?;
                let full_span = span.merge(operand.span());
                Some(Expr::Unary(UnaryOp::Pos, Box::new(operand), full_span))
            }
            TokenKind::Not => {
                let span = self.advance().span;
                let operand = self.parse_pratt_expr(Precedence::Not)?;
                let full_span = span.merge(operand.span());
                Some(Expr::Unary(UnaryOp::Not, Box::new(operand), full_span))
            }
            TokenKind::Fn => self.parse_fn_expr(),
            TokenKind::Fail => {
                // Canon: `fail("message")` creates a recoverable Failure and `fail(err)`
                // repropagates one. The form also terminates a path inside an expression
                // (`return fail(...)`), so it must parse wherever a value is expected.
                let start = self.advance().span;
                let value = if self.check(&TokenKind::LParen) {
                    self.advance();
                    let inner = self.parse_expr()?;
                    self.expect(&TokenKind::RParen, "expected ')' after 'fail' argument")?;
                    inner
                } else {
                    // Bare form `fail err` is the statement spelling and stays accepted.
                    self.parse_expr()?
                };
                let span = start.merge(value.span());
                Some(Expr::Call(CallExpr {
                    callee: Box::new(Expr::Identifier(Ident::new("fail".into(), start))),
                    args: vec![CallArg {
                        name: None,
                        value,
                        span,
                    }],
                    trailing_block: None,
                    span,
                }))
            }
            TokenKind::If => {
                // Inline conditional value form: `if c then a else b`
                let start = self.advance().span;
                let condition = self.parse_expr()?;
                self.expect(
                    &TokenKind::Then,
                    "expected 'then' in conditional value expression",
                )?;
                let then_expr = self.parse_expr()?;
                self.expect(
                    &TokenKind::Else,
                    "expected 'else' in conditional value expression",
                )?;
                let else_expr = self.parse_expr()?;
                let full_span = start.merge(else_expr.span());
                Some(Expr::If(
                    Box::new(condition),
                    Box::new(then_expr),
                    Box::new(else_expr),
                    full_span,
                ))
            }
            TokenKind::Newline | TokenKind::Eof => {
                let span = self
                    .peek_token()
                    .map(|t| t.span)
                    .unwrap_or_else(|| SourceSpan::empty(self.source.len()));
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                        "expected expression, found newline or end of input",
                    )
                    .with_primary_span(self.source, span),
                );
                None
            }
            _ => {
                let tok = self.advance();
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                        format!("unexpected token '{:?}' in expression", tok.kind),
                    )
                    .with_primary_span(self.source, tok.span),
                );
                None
            }
        }
    }

    fn parse_infix_expr(&mut self, left: Expr) -> Option<Expr> {
        let op_tok = self.advance();
        let op_span = op_tok.span;

        match op_tok.kind {
            TokenKind::Plus => self.binary_expr(left, BinaryOp::Add, Precedence::Sum, op_span),
            TokenKind::Minus => self.binary_expr(left, BinaryOp::Sub, Precedence::Sum, op_span),
            TokenKind::Star => self.binary_expr(left, BinaryOp::Mul, Precedence::Product, op_span),
            TokenKind::Slash => self.binary_expr(left, BinaryOp::Div, Precedence::Product, op_span),
            TokenKind::Div => {
                self.binary_expr(left, BinaryOp::IntDiv, Precedence::Product, op_span)
            }
            TokenKind::Percent => {
                self.binary_expr(left, BinaryOp::Mod, Precedence::Product, op_span)
            }
            TokenKind::EqualEqual => {
                self.binary_expr(left, BinaryOp::Equal, Precedence::Comparison, op_span)
            }
            TokenKind::BangEqual => {
                self.binary_expr(left, BinaryOp::NotEqual, Precedence::Comparison, op_span)
            }
            TokenKind::Less => {
                self.binary_expr(left, BinaryOp::Less, Precedence::Comparison, op_span)
            }
            TokenKind::LessEqual => {
                self.binary_expr(left, BinaryOp::LessEqual, Precedence::Comparison, op_span)
            }
            TokenKind::Greater => {
                self.binary_expr(left, BinaryOp::Greater, Precedence::Comparison, op_span)
            }
            TokenKind::GreaterEqual => self.binary_expr(
                left,
                BinaryOp::GreaterEqual,
                Precedence::Comparison,
                op_span,
            ),
            TokenKind::Is => self.binary_expr(left, BinaryOp::Is, Precedence::Comparison, op_span),
            TokenKind::And => self.binary_expr(left, BinaryOp::And, Precedence::And, op_span),
            TokenKind::Or => self.binary_expr(left, BinaryOp::Or, Precedence::Or, op_span),
            TokenKind::OrElse => {
                self.binary_expr(left, BinaryOp::OrElse, Precedence::OrElse, op_span)
            }
            TokenKind::Pipeline => {
                self.binary_expr(left, BinaryOp::Pipeline, Precedence::Pipeline, op_span)
            }
            TokenKind::DotDot => {
                self.binary_expr(left, BinaryOp::Range, Precedence::Range, op_span)
            }

            // Call postfix: `callee(...)`
            TokenKind::LParen => {
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.is_at_end() {
                    // Check for named argument `name = value`
                    let is_named = matches!(self.peek(), TokenKind::Identifier(_))
                        && self.peek_ahead(1) == &TokenKind::Equal;
                    let (arg_name, value) = if is_named {
                        let id = self.parse_ident()?;
                        self.advance(); // consume '='
                        let val = self.parse_expr()?;
                        (Some(id), val)
                    } else {
                        (None, self.parse_expr()?)
                    };
                    let span = arg_name
                        .as_ref()
                        .map(|n| n.span.merge(value.span()))
                        .unwrap_or_else(|| value.span());
                    args.push(CallArg {
                        name: arg_name,
                        value,
                        span,
                    });
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                let end = self
                    .expect(&TokenKind::RParen, "expected ')' after call arguments")?
                    .span;

                // Check for trailing block `do ... end`
                let trailing_block = if self.match_token(&TokenKind::Do) {
                    self.parse_trailing_block()
                } else {
                    None
                };

                let span = if let Some(b) = &trailing_block {
                    left.span().merge(b.span)
                } else {
                    left.span().merge(end)
                };

                Some(Expr::Call(CallExpr {
                    callee: Box::new(left),
                    args,
                    trailing_block,
                    span,
                }))
            }

            // Dot call or field access: `target.field`
            TokenKind::Dot => {
                let field = self.parse_member_name()?;
                let span = left.span().merge(field.span);
                Some(Expr::Dot(Box::new(left), field, span))
            }

            // Safe navigation: `target?.field`
            TokenKind::QuestionDot => {
                let field = self.parse_member_name()?;
                let span = left.span().merge(field.span);
                Some(Expr::QuestionDot(Box::new(left), field, span))
            }

            // Index access: `target[index]`, including half-open slices with omitted
            // bounds (`target[..2]`, `target[2..]`, `target[..]`), which canon allows.
            TokenKind::LBracket => {
                let index_expr = self.parse_index_expr()?;
                let end = self
                    .expect(&TokenKind::RBracket, "expected ']' after index")?
                    .span;
                let span = left.span().merge(end);
                Some(Expr::Index(Box::new(left), Box::new(index_expr), span))
            }

            // Trailing block sugar without parens: `callee do ... end`
            TokenKind::Do => {
                let block = self.parse_trailing_block()?;
                let span = left.span().merge(block.span);
                Some(Expr::Call(CallExpr {
                    callee: Box::new(left),
                    args: Vec::new(),
                    trailing_block: Some(block),
                    span,
                }))
            }

            _ => None,
        }
    }

    fn binary_expr(
        &mut self,
        left: Expr,
        op: BinaryOp,
        prec: Precedence,
        op_span: SourceSpan,
    ) -> Option<Expr> {
        let right = self.parse_pratt_expr(prec)?;
        let span = left.span().merge(right.span());
        let _ = op_span;
        Some(Expr::Binary(op, Box::new(left), Box::new(right), span))
    }

    fn peek_precedence(&self) -> Precedence {
        match self.peek() {
            TokenKind::LParen
            | TokenKind::Dot
            | TokenKind::QuestionDot
            | TokenKind::LBracket
            | TokenKind::Do => Precedence::Call,
            TokenKind::Star | TokenKind::Slash | TokenKind::Div | TokenKind::Percent => {
                Precedence::Product
            }
            TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
            TokenKind::DotDot => Precedence::Range,
            TokenKind::EqualEqual
            | TokenKind::BangEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Is => Precedence::Comparison,
            TokenKind::And => Precedence::And,
            TokenKind::Or => Precedence::Or,
            TokenKind::OrElse => Precedence::OrElse,
            TokenKind::Pipeline => Precedence::Pipeline,
            _ => Precedence::Lowest,
        }
    }

    fn parse_construct_expr(&mut self, target: Ident) -> Option<Expr> {
        let start = target.span;
        self.expect(&TokenKind::LBrace, "expected '{' in struct construction")?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let is_named = matches!(self.peek(), TokenKind::Identifier(_))
                && self.peek_ahead(1) == &TokenKind::Equal;
            let (name, value) = if is_named {
                let id = self.parse_ident()?;
                self.advance(); // consume '='
                let val = self.parse_expr()?;
                (Some(id), val)
            } else {
                (None, self.parse_expr()?)
            };
            let span = name
                .as_ref()
                .map(|n| n.span.merge(value.span()))
                .unwrap_or_else(|| value.span());
            fields.push(ConstructField { name, value, span });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let end = self
            .expect(
                &TokenKind::RBrace,
                "expected '}' to close struct construction",
            )?
            .span;
        Some(Expr::Construct(ConstructExpr {
            target,
            fields,
            span: start.merge(end),
        }))
    }

    fn parse_trailing_block(&mut self) -> Option<TrailingBlock> {
        let start = self.tokens[self.cursor - 1].span; // 'do'
        let mut params = Vec::new();
        if !matches!(self.peek(), TokenKind::Newline) {
            while !self.check(&TokenKind::Newline)
                && !self.check(&TokenKind::End)
                && !self.is_at_end()
            {
                params.push(self.parse_ident()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(&TokenKind::End, "expected 'end' to close trailing block")?
            .span;
        Some(TrailingBlock {
            params,
            body,
            span: start.merge(end_span),
        })
    }

    fn parse_fn_expr(&mut self) -> Option<Expr> {
        let start = self.advance().span; // 'fn'
        self.expect(
            &TokenKind::LParen,
            "expected '(' in anonymous function params",
        )?;
        let params = self.parse_params()?;
        self.expect(
            &TokenKind::RParen,
            "expected ')' after anonymous function params",
        )?;
        let return_type = self.parse_optional_return_type();
        self.skip_newlines();

        let mut body = Vec::new();
        while !self.check(&TokenKind::End) && !self.is_at_end() {
            if let Some(s) = self.parse_stmt() {
                body.push(s);
            }
            self.skip_newlines();
        }
        let end_span = self
            .expect(
                &TokenKind::End,
                "expected 'end' to close anonymous function",
            )?
            .span;
        Some(Expr::Fn(FunctionExpr {
            params,
            return_type,
            body,
            span: start.merge(end_span),
        }))
    }

    fn parse_list_expr(&mut self) -> Option<Expr> {
        let start = self.advance().span; // '['
        let mut items = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.is_at_end() {
            items.push(self.parse_expr()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let end = self
            .expect(&TokenKind::RBracket, "expected ']' after list literal")?
            .span;
        Some(Expr::List(items, start.merge(end)))
    }

    fn parse_dict_expr(&mut self) -> Option<Expr> {
        let start = self.advance().span; // '{'
        let mut pairs = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let key = self.parse_expr()?;
            self.expect(
                &TokenKind::Colon,
                "expected ':' between dictionary key and value",
            )?;
            let val = self.parse_expr()?;
            pairs.push((key, val));
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        let end = self
            .expect(&TokenKind::RBrace, "expected '}' after dict literal")?
            .span;
        Some(Expr::Dict(pairs, start.merge(end)))
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        if let TokenKind::Identifier(name) = self.peek().clone() {
            let span = self.advance().span;
            Some(Ident::new(name, span))
        } else {
            let span = self
                .peek_token()
                .map(|t| t.span)
                .unwrap_or_else(|| SourceSpan::empty(self.source.len()));
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN,
                    format!("expected identifier, found '{:?}'", self.peek()),
                )
                .with_primary_span(self.source, span),
            );
            None
        }
    }

    fn parse_member_name(&mut self) -> Option<Ident> {
        if let Some(tok) = self.peek_token().cloned() {
            if let Some(text) = self.source.slice(tok.span) {
                if !text.is_empty()
                    && (text.starts_with(|c: char| c.is_alphabetic() || c == '_')
                        && text.chars().all(|c| c.is_alphanumeric() || c == '_'))
                {
                    self.advance();
                    return Some(Ident::new(text.to_string(), tok.span));
                }
            }
        }
        self.parse_ident()
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
        | Stmt::Fail(_, span) => *span,
        Stmt::If(s) => s.span,
        Stmt::Match(s) => s.span,
        Stmt::Attempt(s) => s.span,
        Stmt::Fn(f) => f.span,
        Stmt::Expr(e) => e.span(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 1,
    Pipeline,   // |>
    OrElse,     // or_else
    Or,         // or
    And,        // and
    Not,        // not
    Comparison, // == != < <= > >= is
    Range,      // ..
    Sum,        // + -
    Product,    // * / div %
    Prefix,     // - +
    Call,       // . ?. () [] do
}
