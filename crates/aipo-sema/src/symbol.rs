//! Symbols and lexical scope management.

use aipo_source::SourceSpan;
use std::collections::HashMap;

/// Mutability class of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    /// Immutable binding (e.g. `let`, immutable parameter, fixed field).
    Immutable,
    /// Mutable binding (e.g. `var`, mutable parameter `!`, mutable receiver `self!`).
    Mutable,
}

/// Category of semantic symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Variable binding.
    Variable,
    /// Function parameter.
    Parameter,
    /// Function with signature arity (min required args, max allowed args).
    Function {
        /// Minimum required positional arguments.
        min_args: usize,
        /// Maximum allowed arguments.
        max_args: usize,
    },
    /// Struct declaration with declared fields (field name -> is_fixed).
    Struct {
        /// Fields and their immutability (`is_fixed`).
        fields: HashMap<String, bool>,
    },
    /// Interface declaration with required method names and arities (name -> (min, max)).
    Interface {
        /// Required methods and their arity bounds.
        methods: HashMap<String, (usize, usize)>,
    },
}

/// Resolved semantic symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// Identifier name.
    pub name: String,
    /// Symbol category and metadata.
    pub kind: SymbolKind,
    /// Mutability classification.
    pub mutability: Mutability,
    /// Source declaration span.
    pub span: SourceSpan,
}

/// Lexical scope holding local symbols.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Parent scope index in ScopeTree.
    pub parent: Option<usize>,
    /// Symbol table for this scope.
    pub symbols: HashMap<String, Symbol>,
}

/// Hierarchy of lexical scopes.
#[derive(Debug, Default)]
pub struct ScopeTree {
    /// All scopes stored arena-style.
    pub scopes: Vec<Scope>,
}

impl ScopeTree {
    /// Creates a new empty scope tree.
    #[must_use]
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Allocates a new scope with an optional parent.
    pub fn new_scope(&mut self, parent: Option<usize>) -> usize {
        let id = self.scopes.len();
        self.scopes.push(Scope {
            parent,
            symbols: HashMap::new(),
        });
        id
    }

    /// Inserts a symbol into the specified scope. Returns existing symbol if already declared in this scope.
    pub fn insert(&mut self, scope_id: usize, symbol: Symbol) -> Option<Symbol> {
        self.scopes[scope_id]
            .symbols
            .insert(symbol.name.clone(), symbol)
    }

    /// Resolves a symbol by name starting at `scope_id` and walking up to parent scopes.
    #[must_use]
    pub fn lookup(&self, mut scope_id: usize, name: &str) -> Option<&Symbol> {
        loop {
            let scope = &self.scopes[scope_id];
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
            scope_id = scope.parent?;
        }
    }
}
