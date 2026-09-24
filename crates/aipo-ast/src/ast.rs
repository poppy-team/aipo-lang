//! Strongly typed Abstract Syntax Tree (AST) definitions for Aipo V1.

use aipo_source::SourceSpan;
use serde::{Deserialize, Serialize};

/// An identifier with its source location.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ident {
    /// The normalized identifier name.
    pub name: String,
    /// Span in the source text.
    pub span: SourceSpan,
}

impl Ident {
    /// Constructs a new identifier.
    #[must_use]
    pub const fn new(name: String, span: SourceSpan) -> Self {
        Self { name, span }
    }
}

/// Optional type annotation in function parameter or return position (`: Type`, `-> Type`, `T?`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeAnnotation {
    /// The name of the expected type or interface.
    pub name: String,
    /// True if marked nullable `T?` (`T` or `none`).
    pub is_nullable: bool,
    /// Span of the annotation.
    pub span: SourceSpan,
}

/// A parsed program file containing module declarations and executable statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Program {
    /// Module-level declarations (functions, structs, interfaces, imports, exports).
    pub items: Vec<Item>,
    /// Top-level executable statements.
    pub statements: Vec<Stmt>,
    /// Span of the entire program.
    pub span: SourceSpan,
}

/// Top-level structural module items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// Global function declaration.
    Fn(FunctionDecl),
    /// Data structure declaration.
    Struct(StructDecl),
    /// Receiver-associated behavior block.
    Impl(ImplBlock),
    /// Structural interface declaration.
    Interface(InterfaceDecl),
    /// Interface satisfaction declaration.
    Satisfy(SatisfyDecl),
    /// Module import declaration.
    Import(ImportDecl),
    /// Public module export declaration.
    Export(ExportDecl),
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Function name.
    pub name: Ident,
    /// `true` for `async fn`: calling returns a `Task` instead of running.
    pub is_async: bool,
    /// Parameters.
    pub params: Vec<Param>,
    /// Optional return contract.
    pub return_type: Option<TypeAnnotation>,
    /// Function body statements.
    pub body: Vec<Stmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    /// Parameter identifier.
    pub name: Ident,
    /// True if marked mutable `name!` or `self!`.
    pub is_mutable: bool,
    /// Optional contract `name: Type`.
    pub type_annotation: Option<TypeAnnotation>,
    /// Optional default argument value.
    pub default: Option<Expr>,
    /// Source span.
    pub span: SourceSpan,
}

/// Struct declaration defining data fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructDecl {
    /// Struct name.
    pub name: Ident,
    /// Declared fields.
    pub fields: Vec<StructField>,
    /// Source span.
    pub span: SourceSpan,
}

/// Struct field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructField {
    /// Field name.
    pub name: Ident,
    /// True if field is `fixed` (immutable after initialization).
    pub is_fixed: bool,
    /// Optional default value.
    pub default: Option<Expr>,
    /// Source span.
    pub span: SourceSpan,
}

/// Impl block grouping associated functions and hooks for a type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplBlock {
    /// Target type name.
    pub target: Ident,
    /// Optional `init(...)` constructor hook.
    pub init: Option<FunctionDecl>,
    /// Optional `invariant()` validation hook.
    pub invariant: Option<InvariantHook>,
    /// Associated functions.
    pub methods: Vec<FunctionDecl>,
    /// Source span.
    pub span: SourceSpan,
}

/// `invariant()` body containing Boolean validation expressions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantHook {
    /// Boolean conditions required to remain true.
    pub conditions: Vec<Expr>,
    /// Source span.
    pub span: SourceSpan,
}

/// Interface declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceDecl {
    /// Interface name.
    pub name: Ident,
    /// Required method signatures.
    pub methods: Vec<FunctionDecl>,
    /// Source span.
    pub span: SourceSpan,
}

/// `satisfy Type: Interface1, Interface2` structural check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SatisfyDecl {
    /// Target type.
    pub target: Ident,
    /// Satisfied interfaces.
    pub interfaces: Vec<Ident>,
    /// Source span.
    pub span: SourceSpan,
}

/// `import module [as alias] [: names]`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportDecl {
    /// Module name or path segments.
    pub module_name: Vec<Ident>,
    /// Optional alias.
    pub alias: Option<Ident>,
    /// Specific imported names.
    pub names: Vec<Ident>,
    /// Source span.
    pub span: SourceSpan,
}

/// `export name1, name2`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportDecl {
    /// Exported names.
    pub names: Vec<Ident>,
    /// Source span.
    pub span: SourceSpan,
}

/// Binding pattern for `let` and `var`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingPattern {
    /// Single named variable.
    Ident(Ident),
    /// Wildcard discard `_`.
    Discard(SourceSpan),
    /// Positional array destructuring `let [a, b] = ...`
    List(Vec<BindingPattern>, SourceSpan),
    /// Named struct destructuring `let {name, age} = ...`
    Struct(Vec<Ident>, SourceSpan),
}

/// Executable statements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    /// `let pattern = expr`
    Let(BindingPattern, Expr, SourceSpan),
    /// `var pattern = expr`
    Var(BindingPattern, Expr, SourceSpan),
    /// `target = expr`
    Assign(Expr, Expr, SourceSpan),
    /// `target op= expr`
    CompoundAssign(BinaryOp, Expr, Expr, SourceSpan),
    /// `if condition then ... elif ... else ... end`
    If(IfStmt),
    /// `match target when ... else ... end`
    Match(MatchStmt),
    /// `loop ... end`
    Loop(Vec<Stmt>, SourceSpan),
    /// `while condition ... end`
    While(Expr, Vec<Stmt>, SourceSpan),
    /// `repeat count [as i] ... end`
    Repeat(Expr, Option<Ident>, Vec<Stmt>, SourceSpan),
    /// `each item in iterable ... end`
    Each(Vec<Ident>, Expr, Vec<Stmt>, SourceSpan),
    /// `break`
    Break(SourceSpan),
    /// `continue`
    Continue(SourceSpan),
    /// `return [expr]`
    Return(Option<Expr>, SourceSpan),
    /// `fail(expr)`
    Fail(Expr, SourceSpan),
    /// `attempt ... failed [err] ... end`
    Attempt(AttemptStmt),
    /// Local function declaration: `fn name(params) ... end`.
    ///
    /// Canon makes the binding exist when execution reaches the declaration, keeps the name
    /// visible inside the function's own body for recursion, and captures enclosing lexical
    /// bindings automatically.
    Fn(FunctionDecl),
    /// Sequential-await block: `await do ... end` (sugar, desugared before execution).
    AwaitDo(Vec<Stmt>, SourceSpan),
    /// Expression statement.
    Expr(Expr),
}

/// If statement representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IfStmt {
    /// Primary condition.
    pub condition: Expr,
    /// Statements executed if condition is true.
    pub then_branch: Vec<Stmt>,
    /// Zero or more elif branches.
    pub elif_branches: Vec<(Expr, Vec<Stmt>)>,
    /// Optional else branch.
    pub else_branch: Option<Vec<Stmt>>,
    /// Source span.
    pub span: SourceSpan,
}

/// Match statement with value comparison branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchStmt {
    /// Target expression evaluated once.
    pub target: Expr,
    /// When branches: expressions to match against and corresponding statements.
    pub when_arms: Vec<(Vec<Expr>, Vec<Stmt>)>,
    /// Optional else fallback branch.
    pub else_arm: Option<Vec<Stmt>>,
    /// Source span.
    pub span: SourceSpan,
}

/// `attempt ... failed err ... end`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptStmt {
    /// Protected body.
    pub body: Vec<Stmt>,
    /// Optional error binding name.
    pub err_binding: Option<Ident>,
    /// Handler body executed on Failure.
    pub failed_body: Vec<Stmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// Literal values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal
    Int(String),
    /// Floating-point literal
    Float(String),
    /// Boolean literal (`true` or `false`)
    Bool(bool),
    /// String literal with prefix
    String(String, crate::ast::StringPrefix),
    /// `none` literal
    None,
}

/// String prefix for AST string literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StringPrefix {
    /// Standard string `"..."`
    #[default]
    Normal,
    /// Format string `f"..."`
    Format,
    /// Raw string `r"..."`
    Raw,
    /// Format raw string `fr"..."`
    FormatRaw,
}

/// Expressions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal constant value.
    Literal(Literal, SourceSpan),
    /// Variable identifier.
    Identifier(Ident),
    /// Unary operator expression.
    Unary(UnaryOp, Box<Expr>, SourceSpan),
    /// Binary operator expression.
    Binary(BinaryOp, Box<Expr>, Box<Expr>, SourceSpan),
    /// Call expression with positional/named arguments and optional trailing block.
    Call(CallExpr),
    /// Dot navigation `target.field`.
    Dot(Box<Expr>, Ident, SourceSpan),
    /// Safe navigation `target?.field`.
    QuestionDot(Box<Expr>, Ident, SourceSpan),
    /// Indexing `target[index]`.
    Index(Box<Expr>, Box<Expr>, SourceSpan),
    /// Struct instantiation `Type{...}`.
    Construct(ConstructExpr),
    /// Anonymous function literal / closure `fn(...) ... end`.
    Fn(FunctionExpr),
    /// List literal `[a, b, c]`.
    List(Vec<Expr>, SourceSpan),
    /// Dictionary literal `{k: v, ...}`.
    Dict(Vec<(Expr, Expr)>, SourceSpan),
    /// Conditional value form `if condition then a else b`.
    If(Box<Expr>, Box<Expr>, Box<Expr>, SourceSpan),
    /// Suspension point: `await task` drives a `Task` to its value.
    Await(Box<Expr>, SourceSpan),
}

impl Expr {
    /// Returns the source span covering this expression.
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            Self::Literal(_, span)
            | Self::Unary(_, _, span)
            | Self::Binary(_, _, _, span)
            | Self::Dot(_, _, span)
            | Self::QuestionDot(_, _, span)
            | Self::Index(_, _, span)
            | Self::List(_, span)
            | Self::Dict(_, span)
            | Self::If(_, _, _, span)
            | Self::Await(_, span) => *span,
            Self::Identifier(ident) => ident.span,
            Self::Call(call) => call.span,
            Self::Construct(construct) => construct.span,
            Self::Fn(func) => func.span,
        }
    }
}

/// Call expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallExpr {
    /// Callee expression.
    pub callee: Box<Expr>,
    /// Arguments (positional and/or named).
    pub args: Vec<CallArg>,
    /// Optional trailing block `do ... end`.
    pub trailing_block: Option<TrailingBlock>,
    /// Source span.
    pub span: SourceSpan,
}

/// Argument passed to a function or method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallArg {
    /// Optional name for named arguments `foo(name = value)`.
    pub name: Option<Ident>,
    /// Argument value.
    pub value: Expr,
    /// Source span.
    pub span: SourceSpan,
}

/// Trailing block `do [params] ... end`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrailingBlock {
    /// Block closure parameters.
    pub params: Vec<Ident>,
    /// Block statements.
    pub body: Vec<Stmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// Struct instantiation `Type{x = 10, y = 20}` or `Type{10, 20}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructExpr {
    /// Struct type identifier.
    pub target: Ident,
    /// Field initializers.
    pub fields: Vec<ConstructField>,
    /// Source span.
    pub span: SourceSpan,
}

/// Field initializer in `Type{...}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstructField {
    /// Field name (if named).
    pub name: Option<Ident>,
    /// Initializer value.
    pub value: Expr,
    /// Source span.
    pub span: SourceSpan,
}

/// Anonymous function literal / closure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionExpr {
    /// `true` for `async fn(...)`: calling returns a `Task`.
    pub is_async: bool,
    /// Parameters.
    pub params: Vec<Param>,
    /// Optional return contract.
    pub return_type: Option<TypeAnnotation>,
    /// Body statements.
    pub body: Vec<Stmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Negation `-`
    Neg,
    /// Positive `+`
    Pos,
    /// Logical negation `not`
    Not,
}

/// Binary operators with canonical precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `div`
    IntDiv,
    /// `%`
    Mod,

    // Comparison & testing
    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `is`
    Is,
    /// `is ...?`
    IsNullable,

    // Logic & fallbacks
    /// `and`
    And,
    /// `or`
    Or,
    /// `or_else`
    OrElse,

    // Pipeline & ranges
    /// `|>`
    Pipeline,
    /// `..`
    Range,
}
