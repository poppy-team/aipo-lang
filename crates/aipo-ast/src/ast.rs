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
    /// Closed algebraic data type (sum type) declaration.
    Enum(EnumDecl),
    /// Method or `init` hook declared as `Tipo:nome(...)`.
    ///
    /// The association lives in the declaration, so no wrapper block is needed and the
    /// receiver is implicit unless the method needs `var self`.
    Method(MethodDecl),
    /// Structural invariant declared as `Tipo:invariant { ... }`.
    Invariant(TargetInvariant),
    /// Batch association `Tipo::[fn1, fn2]`, promoting free functions to methods.
    Batch(BatchBind),
    /// Structural interface declaration.
    Interface(InterfaceDecl),
    /// Module import declaration.
    Import(ImportDecl),
    /// Public module export declaration.
    Export(ExportDecl),
}

/// Method or `init` hook bound to a type by `Tipo:nome`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodDecl {
    /// Type the method belongs to.
    pub target: Ident,
    /// Method name, or `init` for the construction hook.
    pub function: FunctionDecl,
    /// Source span covering `Tipo:nome(...) { ... }`.
    pub span: SourceSpan,
}

/// Structural invariant bound to a type by `Tipo:invariant`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetInvariant {
    /// Type the invariant belongs to.
    pub target: Ident,
    /// The hook body: Boolean conditions that must hold.
    pub hook: InvariantHook,
    /// Source span covering `Tipo:invariant { ... }`.
    pub span: SourceSpan,
}

/// Batch association `Tipo::[fn1, fn2]`.
///
/// Each name refers to a free function; binding promotes it to a method of `target`,
/// injecting the receiver as the first parameter while keeping the function callable
/// on its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchBind {
    /// Type receiving the functions.
    pub target: Ident,
    /// Free function names being promoted.
    pub functions: Vec<Ident>,
    /// Source span covering `Tipo::[...]`.
    pub span: SourceSpan,
}

/// Function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Compiler directives `#!name ...` applied to this function.
    #[serde(default)]
    pub directives: Vec<Directive>,
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
    /// True if marked mutable (`var name` / `var self`).
    pub is_mutable: bool,
    /// Optional contract `name: Type`.
    pub type_annotation: Option<TypeAnnotation>,
    /// Optional default argument value.
    pub default: Option<Expr>,
    /// Source span.
    pub span: SourceSpan,
}

/// Compiler directive `#!name ...` attached to the next item.
///
/// The `#[serde(default)]` keeps serialized fixtures valid without the field; the parser
/// always fills the vector, even when empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Directive {
    /// Directive name (`test`, `todo`, `deprecated`, `satisfies`, ...).
    pub name: Ident,
    /// Raw argument text after the name (`[tag]`, `"msg"`, or empty).
    pub argument: Option<String>,
    /// Source span covering `#!...`.
    pub span: SourceSpan,
}

/// Struct declaration defining data fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructDecl {
    /// Compiler directives `#!name ...` applied to this struct.
    #[serde(default)]
    pub directives: Vec<Directive>,
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
    /// Optional declared type contract (`campo: Tipo`).
    #[serde(default)]
    pub type_annotation: Option<TypeAnnotation>,
    /// Optional default value.
    pub default: Option<Expr>,
    /// Source span.
    pub span: SourceSpan,
}

/// Enum declaration defining a closed sum type with variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumDecl {
    /// Compiler directives applied to this enum.
    #[serde(default)]
    pub directives: Vec<Directive>,
    /// Enum type name.
    pub name: Ident,
    /// Declared variants.
    pub variants: Vec<EnumVariant>,
    /// Source span.
    pub span: SourceSpan,
}

/// A variant of an enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name.
    pub name: Ident,
    /// Payload carried by this variant.
    pub payload: EnumVariantPayload,
    /// Source span.
    pub span: SourceSpan,
}

/// Payload carried by an enum variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnumVariantPayload {
    /// Unit variant carrying no payload: `Inicial`.
    Unit,
    /// Positional payload: `Desligado(motivo: String)` or `Desligado(String)`.
    Tuple(Vec<EnumTupleField>),
    /// Named fields payload: `Ativo { desde: Int }`.
    Struct(Vec<EnumStructField>),
}

/// Field of a tuple variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumTupleField {
    /// Optional field parameter name (e.g. `motivo` in `Desligado(motivo: String)`).
    pub name: Option<Ident>,
    /// Type contract.
    pub type_annotation: TypeAnnotation,
    /// Source span.
    pub span: SourceSpan,
}

/// Field of a struct variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumStructField {
    /// Field name.
    pub name: Ident,
    /// Type contract.
    pub type_annotation: Option<TypeAnnotation>,
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
    /// When branches: patterns to match, optional guard, and statements.
    pub when_arms: Vec<MatchArm>,
    /// Optional else fallback branch.
    pub else_arm: Option<Vec<Stmt>>,
    /// Source span.
    pub span: SourceSpan,
}

/// One `when` branch of a `match`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchArm {
    /// Alternatives, tested left to right; the first that matches wins.
    pub patterns: Vec<MatchPattern>,
    /// Optional Boolean condition evaluated after a pattern matches
    /// (`when pattern if guard then`). A `false` guard falls through to the next arm.
    pub guard: Option<Expr>,
    /// Statements executed when this arm is selected.
    pub body: Vec<Stmt>,
    /// Source span covering the patterns, the guard and the body.
    pub span: SourceSpan,
}

/// A pattern a `when` arm can match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchPattern {
    /// Equality against the target, as canon's `==` comparison: `when "ready"`.
    Value(Expr),
    /// Struct destructuring `when { name, age }`.
    ///
    /// Every listed name binds to that field of the target and is visible in the
    /// guard and the arm body. The block names only the fields the arm uses.
    Destructure(Vec<Ident>),
    /// Enum variant pattern: `when Estado.Ativo { desde }` or `when Estado.Desligado(motivo)`.
    Variant {
        /// Optional enum type qualification (e.g. `Estado` in `Estado.Ativo`).
        enum_name: Option<Ident>,
        /// Variant name (e.g. `Ativo`).
        variant_name: Ident,
        /// Payload binding pattern.
        payload: VariantPatternPayload,
        /// Source span covering the variant pattern.
        span: SourceSpan,
    },
}

/// Payload binding in a variant pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariantPatternPayload {
    /// No payload: `when Estado.Inicial`.
    Unit,
    /// Positional bindings: `when Estado.Desligado(motivo)`.
    Tuple(Vec<Ident>),
    /// Named bindings: `when Estado.Ativo { desde }`.
    Struct(Vec<Ident>),
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
    /// Failure propagation: `expr?` evaluates `expr`, propagates a `Failure`
    /// to the enclosing handler or caller, otherwise yields the value.
    Try(Box<Expr>, SourceSpan),
    /// Functional struct update: `base with { field: value, ... }`.
    ///
    /// Produces a **new** instance: the base is copied and the named fields are
    /// replaced. The base itself is never mutated, and fields omitted from the
    /// block keep the value they had, so the block lists only what changes.
    With(Box<Expr>, WithExpr),
}

/// Functional struct update `base with { ... }`.
///
/// The field list mirrors [`ConstructExpr`]'s so the frontend validates both with
/// the same rules; only the meaning differs, since the block is a sparse set of
/// overrides rather than a full initializer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithExpr {
    /// Fields being replaced, in source order.
    pub fields: Vec<ConstructField>,
    /// Span of the whole update, from the base through the closing brace.
    pub span: SourceSpan,
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
            | Self::Await(_, span)
            | Self::Try(_, span) => *span,
            Self::Identifier(ident) => ident.span,
            Self::Call(call) => call.span,
            Self::Construct(construct) => construct.span,
            Self::Fn(func) => func.span,
            Self::With(_, with) => with.span,
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
    /// `//`
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
