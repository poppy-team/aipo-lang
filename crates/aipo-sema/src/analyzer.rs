//! Semantic analysis and verification pass.

use crate::prelude::PreludeSurface;
use crate::symbol::{Mutability, ScopeTree, Symbol, SymbolKind};
use aipo_ast::{Literal, TypeAnnotation};
use aipo_diagnostics::{Diagnostic, DiagnosticCode};
use aipo_hir::*;
use aipo_source::{Source, SourceSpan};
use std::collections::{HashMap, HashSet};

/// Core categories a written contract can name.
///
/// The list mirrors what the runtime can decide about a value (`aipo-vm::TypeTag`); a contract
/// naming anything else is left to the runtime, because a provable category test is the only
/// thing a pre-execution report may act on.
const CORE_CONTRACT_CATEGORIES: [&str; 9] = [
    "Bool", "Int", "Float", "Byte", "String", "List", "Dict", "Bytes", "Range",
];

/// Canonical category of a literal, when the category is provable from the literal itself.
fn literal_category(literal: &Literal) -> Option<&'static str> {
    match literal {
        Literal::Int(_) => Some("Int"),
        Literal::Float(_) => Some("Float"),
        Literal::Bool(_) => Some("Bool"),
        Literal::String(_, _) => Some("String"),
        // `none` belongs to no category: it only satisfies a nullable contract.
        Literal::None => None,
    }
}

/// Returns whether a literal expression provably cannot satisfy `annotation`.
///
/// Only a literal has a provable category, so a core-type contract naming a different category is
/// a mismatch before execution; a `struct`/interface contract, or any non-literal expression,
/// stays a runtime contract fault.
fn literal_violates_contract(expression: &HirExpr, annotation: &TypeAnnotation) -> bool {
    let HirExpr::Literal(literal, _) = expression else {
        return false;
    };
    match literal_category(literal) {
        // `none` satisfies only a contract that accepts it.
        None => !annotation.is_nullable && annotation.name != "none",
        // Only a core-type contract is provable from the literal itself: a `struct` or interface
        // name needs the runtime, which decides it at the boundary a fault.
        Some(category) => {
            CORE_CONTRACT_CATEGORIES.contains(&annotation.name.as_str())
                && annotation.name != category
        }
    }
}

/// Renders a contract the way it is written (`Int`, `Int?`).
fn contract_label(annotation: &TypeAnnotation) -> String {
    if annotation.is_nullable {
        format!("{}?", annotation.name)
    } else {
        annotation.name.clone()
    }
}

/// One declared parameter contract, aligned with its declaration position.
#[derive(Debug, Clone)]
struct ParameterContract {
    /// Parameter name, used to match a named argument.
    name: String,
    /// Optional written contract.
    annotation: Option<TypeAnnotation>,
}

/// Written signature contracts of a declared function.
#[derive(Debug, Clone, Default)]
struct DeclaredContract {
    /// Parameter contracts in declaration order.
    params: Vec<ParameterContract>,
}

impl DeclaredContract {
    /// Collects the parameter contracts of a declaration.
    ///
    /// A return contract is checked where the `return` is written (it belongs to the enclosing
    /// function body), so only the parameter layout is needed at the call site.
    fn of(function: &HirFunctionDecl) -> Self {
        Self {
            params: function
                .params
                .iter()
                .map(|param| ParameterContract {
                    name: param.name.clone(),
                    annotation: param.type_annotation.clone(),
                })
                .collect(),
        }
    }
}

/// Resulting semantic facts produced by the analyzer.
#[derive(Debug, Default)]
pub struct SemanticFacts {
    /// Scope hierarchy.
    pub scopes: ScopeTree,
}

/// Semantic analyzer walking the HIR and collecting diagnostics.
pub struct SemanticAnalyzer<'a> {
    source: &'a Source,
    facts: SemanticFacts,
    current_scope: usize,
    diagnostics: Vec<Diagnostic>,
    impl_methods: HashMap<String, HashMap<String, (usize, usize)>>, // struct -> (method -> (min, max))
    current_fn_is_mut: bool,
    /// Written signature contracts of every declared function, by name.
    fn_contracts: HashMap<String, DeclaredContract>,
    /// Return contract of the function currently being analyzed, checked at every `return`.
    current_return_contract: Option<TypeAnnotation>,
    /// Root scope of the module, which owns the module-level bindings.
    root_scope: usize,
    /// Names declared by a top-level `let`/`var`, collected before any body is analyzed.
    ///
    /// Canon gives every function of a module access to the module's bindings, so the names
    /// exist in the module scope from the start; the executable flow is what stays textual,
    /// which is why reaching a declaration is tracked separately.
    module_bindings: HashSet<String>,
    /// Module bindings the executable flow has already reached, in textual order.
    reached_bindings: HashSet<String>,
    /// `true` while the module's executable statements are analyzed.
    ///
    /// Function, method and closure bodies are deferred code: they run after the module has
    /// initialized, so they see every module binding and never depend on textual position.
    in_module_flow: bool,
}

impl<'a> SemanticAnalyzer<'a> {
    /// Constructs a new semantic analyzer with only the language's own surface.
    #[must_use]
    pub fn new(source: &'a Source) -> Self {
        Self::with_prelude(source, &PreludeSurface::fundamental())
    }

    /// Constructs a semantic analyzer with an explicit global surface.
    ///
    /// Callers that embed the standard library pass the surface derived from the same
    /// registration the VM will execute with, so `check` and `run` agree on what exists.
    #[must_use]
    pub fn with_prelude(source: &'a Source, surface: &PreludeSurface) -> Self {
        let mut facts = SemanticFacts::default();
        let root = facts.scopes.new_scope(None);

        let mut analyzer = Self {
            source,
            facts,
            current_scope: root,
            diagnostics: Vec::new(),
            impl_methods: HashMap::new(),
            current_fn_is_mut: false,
            fn_contracts: HashMap::new(),
            current_return_contract: None,
            root_scope: root,
            module_bindings: HashSet::new(),
            reached_bindings: HashSet::new(),
            in_module_flow: false,
        };

        analyzer.register_surface(surface);
        analyzer
    }

    /// Installs every name of the configured surface into the root scope.
    fn register_surface(&mut self, surface: &PreludeSurface) {
        let mut entries: Vec<(&String, &SymbolKind)> = surface.iter().collect();
        entries.sort_by(|left, right| left.0.cmp(right.0));

        for (name, kind) in entries {
            self.facts.scopes.insert(
                self.current_scope,
                Symbol {
                    name: name.clone(),
                    kind: kind.clone(),
                    mutability: Mutability::Immutable,
                    span: aipo_source::SourceSpan::empty(0),
                },
            );
        }
    }

    /// Performs semantic analysis on an `HirProgram`.
    pub fn analyze(mut self, program: &HirProgram) -> (SemanticFacts, Vec<Diagnostic>) {
        // Pass 1: Declare top-level items
        for item in &program.items {
            self.declare_item(item);
        }

        // Pass 2: Declare the module's own bindings. Functions, methods and closures resolve
        // them by name, and canon makes the whole module scope visible to them, so this runs
        // before any body is walked.
        self.declare_module_bindings(program);

        // Pass 3: Collect impl methods and verify satisfy declarations
        for item in &program.items {
            if let HirItem::Impl(impl_block) = item {
                let methods = self
                    .impl_methods
                    .entry(impl_block.target.clone())
                    .or_default();
                for m in &impl_block.methods {
                    let min_args = m.params.iter().filter(|p| p.default.is_none()).count();
                    let max_args = m.params.len();
                    methods.insert(m.name.clone(), (min_args, max_args));
                }
            }
        }

        for item in &program.items {
            if let HirItem::Satisfy(sat) = item {
                self.verify_satisfy(sat);
            }
        }

        // Pass 4: Analyze item bodies (deferred code, so no textual-order tracking)
        for item in &program.items {
            self.analyze_item_body(item);
        }

        // Pass 5: Analyze top-level statements in textual order
        self.in_module_flow = true;
        for stmt in &program.statements {
            self.analyze_stmt(stmt);
        }
        self.in_module_flow = false;

        (self.facts, self.diagnostics)
    }

    fn declare_item(&mut self, item: &HirItem) {
        match item {
            HirItem::Fn(f) => {
                let min_args = f.params.iter().filter(|p| p.default.is_none()).count();
                let max_args = f.params.len();
                let sym = Symbol {
                    name: f.name.clone(),
                    kind: SymbolKind::Function { min_args, max_args },
                    mutability: Mutability::Immutable,
                    span: f.span,
                };
                if self.facts.scopes.insert(self.current_scope, sym).is_some() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE,
                            format!("function '{}' is already declared in this scope", f.name),
                        )
                        .with_primary_span(self.source, f.span),
                    );
                }
                // Canon reports a provable contract incompatibility before execution, so the
                // written annotations are collected here for the call and return checks.
                self.fn_contracts
                    .insert(f.name.clone(), DeclaredContract::of(f));
            }
            HirItem::Struct(s) => {
                let mut fields = HashMap::new();
                for f in &s.fields {
                    fields.insert(f.name.clone(), f.is_fixed);
                }
                let sym = Symbol {
                    name: s.name.clone(),
                    kind: SymbolKind::Struct { fields },
                    mutability: Mutability::Immutable,
                    span: s.span,
                };
                if self.facts.scopes.insert(self.current_scope, sym).is_some() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE,
                            format!("struct '{}' is already declared in this scope", s.name),
                        )
                        .with_primary_span(self.source, s.span),
                    );
                }
            }
            HirItem::Interface(i) => {
                let mut methods = HashMap::new();
                for m in &i.methods {
                    let min_args = m.params.iter().filter(|p| p.default.is_none()).count();
                    let max_args = m.params.len();
                    methods.insert(m.name.clone(), (min_args, max_args));
                }
                let sym = Symbol {
                    name: i.name.clone(),
                    kind: SymbolKind::Interface { methods },
                    mutability: Mutability::Immutable,
                    span: i.span,
                };
                if self.facts.scopes.insert(self.current_scope, sym).is_some() {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE,
                            format!("interface '{}' is already declared in this scope", i.name),
                        )
                        .with_primary_span(self.source, i.span),
                    );
                }
            }
            HirItem::Import(imp) => {
                for name in &imp.names {
                    let sym = Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        mutability: Mutability::Immutable,
                        span: imp.span,
                    };
                    self.facts.scopes.insert(self.current_scope, sym);
                }
                if let Some(alias) = &imp.alias {
                    let sym = Symbol {
                        name: alias.clone(),
                        kind: SymbolKind::Variable,
                        mutability: Mutability::Immutable,
                        span: imp.span,
                    };
                    self.facts.scopes.insert(self.current_scope, sym);
                }
            }
            HirItem::Impl(_) | HirItem::Satisfy(_) | HirItem::Export(_) => {}
        }
    }

    fn verify_satisfy(&mut self, sat: &HirSatisfyDecl) {
        let target_sym = self.facts.scopes.lookup(self.current_scope, &sat.target);
        if target_sym.is_none() {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                    format!("unknown satisfy target type '{}'", sat.target),
                )
                .with_primary_span(self.source, sat.span),
            );
            return;
        }

        let struct_methods = self
            .impl_methods
            .get(&sat.target)
            .cloned()
            .unwrap_or_default();

        for iface_name in &sat.interfaces {
            let iface_sym = self.facts.scopes.lookup(self.current_scope, iface_name);
            match iface_sym {
                Some(Symbol {
                    kind: SymbolKind::Interface { methods },
                    ..
                }) => {
                    for (method_name, (req_min, req_max)) in methods {
                        if let Some((prov_min, prov_max)) = struct_methods.get(method_name) {
                            if *prov_min > *req_min || *prov_max < *req_max {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        DiagnosticCode::AIPO_SEM_ARITY_MISMATCH,
                                        format!(
                                            "method '{}' on '{}' does not match interface '{}' arity contract",
                                            method_name, sat.target, iface_name
                                        ),
                                    )
                                    .with_primary_span(self.source, sat.span),
                                );
                            }
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                                    format!(
                                        "struct '{}' satisfies '{}' but is missing method '{}'",
                                        sat.target, iface_name, method_name
                                    ),
                                )
                                .with_primary_span(self.source, sat.span),
                            );
                        }
                    }
                }
                _ => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                            format!("unknown interface '{}' in satisfy declaration", iface_name),
                        )
                        .with_primary_span(self.source, sat.span),
                    );
                }
            }
        }
    }

    fn analyze_item_body(&mut self, item: &HirItem) {
        match item {
            HirItem::Fn(f) => self.analyze_function(f),
            HirItem::Impl(i) => {
                if let Some(init) = &i.init {
                    self.analyze_function(init);
                }
                for method in &i.methods {
                    self.analyze_function(method);
                }
            }
            _ => {}
        }
    }

    fn analyze_function(&mut self, f: &HirFunctionDecl) {
        let parent = self.current_scope;
        let fn_scope = self.facts.scopes.new_scope(Some(parent));
        self.current_scope = fn_scope;

        let prev_fn_is_mut = self.current_fn_is_mut;
        self.current_fn_is_mut = f.params.iter().any(|p| p.is_self && p.is_mut);
        let prev_return_contract = self.current_return_contract.clone();
        self.current_return_contract = f.return_type.clone();

        for p in &f.params {
            let mutability = if p.is_mut {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };
            let sym = Symbol {
                name: p.name.clone(),
                kind: SymbolKind::Parameter,
                mutability,
                span: p.span,
            };
            if self.facts.scopes.insert(self.current_scope, sym).is_some() {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE,
                        format!("parameter '{}' is declared more than once", p.name),
                    )
                    .with_primary_span(self.source, p.span),
                );
            }
        }

        for stmt in &f.body {
            self.analyze_stmt(stmt);
        }

        self.current_fn_is_mut = prev_fn_is_mut;
        self.current_return_contract = prev_return_contract;
        self.current_scope = parent;
    }

    fn analyze_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let(name, expr, span) => {
                // Canon keeps a binding out of its own initializer, so the expression is walked
                // before the name becomes visible to the executable flow.
                self.analyze_expr(expr);
                self.declare_binding(name, Mutability::Immutable, *span);
            }
            HirStmt::Var(name, expr, span) => {
                self.analyze_expr(expr);
                self.declare_binding(name, Mutability::Mutable, *span);
            }
            HirStmt::Assign(target, value, span) => {
                self.analyze_expr(value);
                self.check_assignment_target(target, *span);
            }
            HirStmt::CompoundAssign(_, target, value, span) => {
                self.analyze_expr(value);
                self.check_assignment_target(target, *span);
            }
            HirStmt::If(s) => {
                self.analyze_expr(&s.condition);
                self.with_block_scope(|this| {
                    for st in &s.then_branch {
                        this.analyze_stmt(st);
                    }
                });
                for (cond, body) in &s.elif_branches {
                    self.analyze_expr(cond);
                    self.with_block_scope(|this| {
                        for st in body {
                            this.analyze_stmt(st);
                        }
                    });
                }
                if let Some(else_branch) = &s.else_branch {
                    self.with_block_scope(|this| {
                        for st in else_branch {
                            this.analyze_stmt(st);
                        }
                    });
                }
            }
            HirStmt::Match(s) => {
                self.analyze_expr(&s.target);
                for (patterns, body) in &s.when_arms {
                    for p in patterns {
                        self.analyze_expr(p);
                    }
                    self.with_block_scope(|this| {
                        for st in body {
                            this.analyze_stmt(st);
                        }
                    });
                }
                if let Some(else_arm) = &s.else_arm {
                    self.with_block_scope(|this| {
                        for st in else_arm {
                            this.analyze_stmt(st);
                        }
                    });
                }
            }
            HirStmt::Loop(body, _) => {
                self.with_block_scope(|this| {
                    for st in body {
                        this.analyze_stmt(st);
                    }
                });
            }
            HirStmt::While(cond, body, _) => {
                self.analyze_expr(cond);
                self.with_block_scope(|this| {
                    for st in body {
                        this.analyze_stmt(st);
                    }
                });
            }
            HirStmt::Repeat(count, alias, body, span) => {
                self.analyze_expr(count);
                self.with_block_scope(|this| {
                    if let Some(name) = alias {
                        this.facts.scopes.insert(
                            this.current_scope,
                            Symbol {
                                name: name.clone(),
                                kind: SymbolKind::Variable,
                                mutability: Mutability::Immutable,
                                span: *span,
                            },
                        );
                    }
                    for st in body {
                        this.analyze_stmt(st);
                    }
                });
            }
            HirStmt::Each(vars, iter, body, span) => {
                self.analyze_expr(iter);
                self.with_block_scope(|this| {
                    for v in vars {
                        this.facts.scopes.insert(
                            this.current_scope,
                            Symbol {
                                name: v.clone(),
                                kind: SymbolKind::Variable,
                                mutability: Mutability::Immutable,
                                span: *span,
                            },
                        );
                    }
                    for st in body {
                        this.analyze_stmt(st);
                    }
                });
            }
            HirStmt::Break(_) | HirStmt::Continue(_) => {}
            HirStmt::Return(expr, span) => {
                if let Some(e) = expr {
                    self.analyze_expr(e);
                    self.check_return_contract(e, *span);
                }
            }
            HirStmt::Fail(expr, _) => self.analyze_expr(expr),
            HirStmt::Attempt(s) => {
                self.with_block_scope(|this| {
                    for st in &s.body {
                        this.analyze_stmt(st);
                    }
                });
                self.with_block_scope(|this| {
                    if let Some(err) = &s.error_binding {
                        this.facts.scopes.insert(
                            this.current_scope,
                            Symbol {
                                name: err.clone(),
                                kind: SymbolKind::Variable,
                                mutability: Mutability::Immutable,
                                span: s.span,
                            },
                        );
                    }
                    for st in &s.handler {
                        this.analyze_stmt(st);
                    }
                });
            }
            HirStmt::FnDecl(f) => {
                // Canon: a local function's binding exists when execution reaches the
                // declaration and stays visible inside its own body for recursion, so the
                // name is declared before the body is walked. Enclosing bindings stay
                // visible (lexical capture), but the body is deferred code: `let` bindings
                // of the enclosing flow that were not reached yet must not leak in, which
                // the block scope + module-flow reset already handle for closures.
                self.declare_binding(&f.name, Mutability::Immutable, f.span);
                let prev_return_contract = self.current_return_contract.clone();
                let prev_module_flow = self.in_module_flow;
                self.current_return_contract = f.return_type.clone();
                self.in_module_flow = false;
                self.with_block_scope(|this| {
                    for p in &f.params {
                        let mutability = if p.is_mut {
                            Mutability::Mutable
                        } else {
                            Mutability::Immutable
                        };
                        this.facts.scopes.insert(
                            this.current_scope,
                            Symbol {
                                name: p.name.clone(),
                                kind: SymbolKind::Parameter,
                                mutability,
                                span: p.span,
                            },
                        );
                    }
                    for st in &f.body {
                        this.analyze_stmt(st);
                    }
                });
                self.current_return_contract = prev_return_contract;
                self.in_module_flow = prev_module_flow;
            }
            HirStmt::Expr(expr) => self.analyze_expr(expr),
        }
    }

    fn check_assignment_target(&mut self, target: &HirExpr, span: aipo_source::SourceSpan) {
        match target {
            HirExpr::Identifier(name, id_span) => {
                let symbol = self
                    .name_is_visible(name)
                    .then(|| self.facts.scopes.lookup(self.current_scope, name))
                    .flatten();
                match symbol {
                    Some(sym) if sym.mutability == Mutability::Immutable => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::AIPO_SEM_READONLY_MUTATION,
                                format!("cannot reassign to immutable binding '{}'", name),
                            )
                            .with_primary_span(self.source, *id_span),
                        );
                    }
                    Some(_) => {}
                    None => {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                                format!("unknown identifier '{}' in assignment", name),
                            )
                            .with_primary_span(self.source, *id_span),
                        );
                    }
                }
            }
            HirExpr::Dot(base, member, dot_span) => {
                self.analyze_expr(base);
                if let HirExpr::Identifier(base_name, _) = &**base {
                    if base_name == "self" && !self.current_fn_is_mut {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::AIPO_SEM_READONLY_MUTATION,
                                format!(
                                    "cannot mutate field '{}' through immutable receiver 'self'; method requires 'self!'",
                                    member
                                ),
                            )
                            .with_primary_span(self.source, *dot_span),
                        );
                    }
                }
            }
            HirExpr::Index(base, idx, _) => {
                self.analyze_expr(base);
                self.analyze_expr(idx);
            }
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::AIPO_PARSE_INVALID_TARGET,
                        "invalid target in assignment",
                    )
                    .with_primary_span(self.source, span),
                );
            }
        }
    }

    /// Reports contract mismatches the analyzer can prove from literal arguments.
    ///
    /// Canon reports a provable incompatibility before execution and leaves an unprovable one as
    /// a runtime contract fault, so only literal categories are judged here. Arguments follow the
    /// declaration: positional arguments first, then named ones matched by parameter name.
    fn check_argument_contracts(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
        contract: &DeclaredContract,
    ) {
        let positional = args.iter().take_while(|arg| arg.name.is_none()).count();
        for (index, param) in contract.params.iter().enumerate() {
            let Some(annotation) = &param.annotation else {
                continue;
            };
            let argument = if index < positional {
                args.get(index)
            } else {
                args.iter()
                    .skip(positional)
                    .find(|arg| arg.name.as_deref() == Some(param.name.as_str()))
            };
            let Some(argument) = argument else {
                continue;
            };
            if literal_violates_contract(&argument.value, annotation) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC,
                        format!(
                            "argument for parameter '{}' of '{}' cannot satisfy contract '{}'",
                            param.name,
                            callee,
                            contract_label(annotation)
                        ),
                    )
                    .with_primary_span(self.source, argument.value.span()),
                );
            }
        }
    }

    /// Reports a return value that provably cannot satisfy the declared return contract.
    fn check_return_contract(&mut self, expression: &HirExpr, span: SourceSpan) {
        let Some(annotation) = self.current_return_contract.clone() else {
            return;
        };
        if literal_violates_contract(expression, &annotation) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC,
                    format!(
                        "return value cannot satisfy contract '{}'",
                        contract_label(&annotation)
                    ),
                )
                .with_primary_span(self.source, span),
            );
        }
    }

    /// Declares the bindings a top-level `let`/`var` introduces, in module scope.
    ///
    /// The executable flow still sees them in textual order, so this only records the names;
    /// [`SemanticAnalyzer::declare_binding`] marks each one reached when the flow arrives at its
    /// declaration, and an early reference is reported as an unknown name.
    fn declare_module_bindings(&mut self, program: &HirProgram) {
        for stmt in &program.statements {
            let (name, mutability, span) = match stmt {
                HirStmt::Let(name, _, span) => (name, Mutability::Immutable, *span),
                HirStmt::Var(name, _, span) => (name, Mutability::Mutable, *span),
                _ => continue,
            };
            self.module_bindings.insert(name.clone());
            self.facts.scopes.insert(
                self.root_scope,
                Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    mutability,
                    span,
                },
            );
        }
    }

    /// Declares a `let`/`var` binding in the scope the statement is written in.
    fn declare_binding(&mut self, name: &str, mutability: Mutability, span: SourceSpan) {
        let is_module_flow = self.in_module_flow && self.current_scope == self.root_scope;
        if is_module_flow && self.module_bindings.contains(name) {
            // The name already exists in the module scope, so this statement is where the
            // executable flow reaches it — not a redeclaration, unless it was reached twice.
            if !self.reached_bindings.insert(name.to_string()) {
                self.report_redeclared_binding(name, span);
            }
            return;
        }

        if self.facts.scopes.scopes[self.current_scope]
            .symbols
            .contains_key(name)
        {
            self.report_redeclared_binding(name, span);
        } else {
            self.facts.scopes.insert(
                self.current_scope,
                Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Variable,
                    mutability,
                    span,
                },
            );
        }
    }

    fn report_redeclared_binding(&mut self, name: &str, span: SourceSpan) {
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE,
                format!("binding '{}' is already declared in this scope", name),
            )
            .with_primary_span(self.source, span),
        );
    }

    /// Reports whether `name` resolves at the current point of the program.
    ///
    /// A module binding is visible to every function of the module, but the executable flow
    /// only reaches it at its declaration, which keeps canon's textual-order rule for `let`/`var`.
    fn name_is_visible(&self, name: &str) -> bool {
        if self.in_module_flow
            && self.module_bindings.contains(name)
            && !self.reached_bindings.contains(name)
        {
            return false;
        }
        self.facts.scopes.lookup(self.current_scope, name).is_some()
    }

    fn analyze_expr(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Literal(_, _) => {}
            HirExpr::Identifier(name, span) => {
                if !self.name_is_visible(name) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                            format!("unknown identifier '{}'", name),
                        )
                        .with_primary_span(self.source, *span),
                    );
                }
            }
            HirExpr::Unary(_, inner, _) => self.analyze_expr(inner),
            HirExpr::Binary(_, left, right, _) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            HirExpr::Call(callee, args, span) => {
                self.analyze_expr(callee);
                for arg in args {
                    self.analyze_expr(&arg.value);
                }

                if let HirExpr::Identifier(name, _) = &**callee {
                    if let Some(Symbol {
                        kind: SymbolKind::Function { min_args, max_args },
                        ..
                    }) = self.facts.scopes.lookup(self.current_scope, name)
                    {
                        if args.len() < *min_args || args.len() > *max_args {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    DiagnosticCode::AIPO_SEM_ARITY_MISMATCH,
                                    format!(
                                        "function '{}' expected between {} and {} arguments, found {}",
                                        name, min_args, max_args, args.len()
                                    ),
                                )
                                .with_primary_span(self.source, *span),
                            );
                        }
                    }
                    if let Some(contract) = self.fn_contracts.get(name).cloned() {
                        self.check_argument_contracts(name, args, &contract);
                    }
                }
            }
            HirExpr::Dot(target, _, _) | HirExpr::QuestionDot(target, _, _) => {
                self.analyze_expr(target);
            }
            HirExpr::Index(target, idx, _) => {
                self.analyze_expr(target);
                self.analyze_expr(idx);
            }
            HirExpr::List(items, _) => {
                for i in items {
                    self.analyze_expr(i);
                }
            }
            HirExpr::Dict(pairs, _) => {
                for (k, v) in pairs {
                    self.analyze_expr(k);
                    self.analyze_expr(v);
                }
            }
            HirExpr::Construct(target, fields, span) => {
                if self
                    .facts
                    .scopes
                    .lookup(self.current_scope, target)
                    .is_none()
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
                            format!("unknown struct type '{}' in construction", target),
                        )
                        .with_primary_span(self.source, *span),
                    );
                }
                for (_, f_expr) in fields {
                    self.analyze_expr(f_expr);
                }
            }
            HirExpr::Fn(f) => {
                // A closure owns its own return contract: an inner `return` must not be judged
                // against the contract of the function that declares the closure. Its body is
                // deferred code too, so it resolves module bindings without textual order.
                let prev_return_contract = self.current_return_contract.clone();
                let prev_module_flow = self.in_module_flow;
                self.current_return_contract = f.return_type.clone();
                self.in_module_flow = false;
                self.with_block_scope(|this| {
                    for p in &f.params {
                        let mutability = if p.is_mut {
                            Mutability::Mutable
                        } else {
                            Mutability::Immutable
                        };
                        this.facts.scopes.insert(
                            this.current_scope,
                            Symbol {
                                name: p.name.clone(),
                                kind: SymbolKind::Parameter,
                                mutability,
                                span: p.span,
                            },
                        );
                    }
                    for st in &f.body {
                        this.analyze_stmt(st);
                    }
                });
                self.current_return_contract = prev_return_contract;
                self.in_module_flow = prev_module_flow;
            }
            HirExpr::If(c, t, e, _) => {
                self.analyze_expr(c);
                self.analyze_expr(t);
                self.analyze_expr(e);
            }
            HirExpr::OrElse(l, r, _) => {
                self.analyze_expr(l);
                self.analyze_expr(r);
            }
        }
    }

    fn with_block_scope<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let parent = self.current_scope;
        let block_scope = self.facts.scopes.new_scope(Some(parent));
        self.current_scope = block_scope;
        f(self);
        self.current_scope = parent;
    }
}
