//! High-Level Intermediate Representation (HIR) nodes for Aipo.

use aipo_ast::{BinaryOp, Literal, TypeAnnotation, UnaryOp};
use aipo_source::SourceSpan;
use serde::{Deserialize, Serialize};

/// High-Level Intermediate Representation of an entire module or script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirProgram {
    /// Structural items (functions, structs, impls, interfaces, imports, exports).
    pub items: Vec<HirItem>,
    /// Executable statements.
    pub statements: Vec<HirStmt>,
    /// Full module span.
    pub span: SourceSpan,
}

/// Structural declarations in HIR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HirItem {
    /// Function declaration.
    Fn(HirFunctionDecl),
    /// Struct declaration.
    Struct(HirStructDecl),
    /// Impl block.
    Impl(HirImplBlock),
    /// Interface declaration.
    Interface(HirInterfaceDecl),
    /// Interface satisfaction.
    Satisfy(HirSatisfyDecl),
    /// Import declaration.
    Import(HirImportDecl),
    /// Export declaration.
    Export(HirExportDecl),
}

/// HIR function declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirFunctionDecl {
    /// Function name.
    pub name: String,
    /// Parameters.
    pub params: Vec<HirParam>,
    /// Optional return type contract.
    pub return_type: Option<TypeAnnotation>,
    /// Function body statements.
    pub body: Vec<HirStmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR function parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirParam {
    /// Parameter name.
    pub name: String,
    /// Mutability marker `!`.
    pub is_mut: bool,
    /// Receiver marker `self` / `self!`.
    pub is_self: bool,
    /// Optional type contract.
    pub type_annotation: Option<TypeAnnotation>,
    /// Optional default value expression.
    pub default: Option<HirExpr>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR struct declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirStructDecl {
    /// Struct name.
    pub name: String,
    /// Field declarations.
    pub fields: Vec<HirStructField>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR struct field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirStructField {
    /// Field name.
    pub name: String,
    /// Whether field is immutable (`fixed`).
    pub is_fixed: bool,
    /// Optional default value.
    pub default: Option<HirExpr>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR impl block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirImplBlock {
    /// Target type name.
    pub target: String,
    /// Optional init constructor hook.
    pub init: Option<HirFunctionDecl>,
    /// Optional invariant hook, lowered to a `self`-taking predicate.
    ///
    /// Canon combines the hook's lines with `and` and evaluates the result at the end of
    /// construction (and at guarded mutation points), so the hook is represented as a function
    /// returning `Bool` rather than as raw expressions: everything after lowering sees the same
    /// shape as any other predicate.
    pub invariant: Option<HirFunctionDecl>,
    /// Associated methods.
    pub methods: Vec<HirFunctionDecl>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR interface declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirInterfaceDecl {
    /// Interface name.
    pub name: String,
    /// Required method signatures.
    pub methods: Vec<HirFunctionDecl>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR satisfy declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirSatisfyDecl {
    /// Struct type name.
    pub target: String,
    /// Interfaces satisfied.
    pub interfaces: Vec<String>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR import declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirImportDecl {
    /// Module name.
    pub module_name: String,
    /// Optional alias.
    pub alias: Option<String>,
    /// Imported names.
    pub names: Vec<String>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR export declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirExportDecl {
    /// Exported names.
    pub names: Vec<String>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR executable statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HirStmt {
    /// `let name = expr`
    Let(String, HirExpr, SourceSpan),
    /// `var name = expr`
    Var(String, HirExpr, SourceSpan),
    /// `target = expr`
    Assign(HirExpr, HirExpr, SourceSpan),
    /// `target op= expr`
    CompoundAssign(BinaryOp, HirExpr, HirExpr, SourceSpan),
    /// `if cond then ... elif ... else ... end`
    If(HirIfStmt),
    /// `match target when ... else ... end`
    Match(HirMatchStmt),
    /// `loop ... end`
    Loop(Vec<HirStmt>, SourceSpan),
    /// `while cond ... end`
    While(HirExpr, Vec<HirStmt>, SourceSpan),
    /// `repeat count [as i] ... end`
    Repeat(HirExpr, Option<String>, Vec<HirStmt>, SourceSpan),
    /// `each item in iter ... end`
    Each(Vec<String>, HirExpr, Vec<HirStmt>, SourceSpan),
    /// `break`
    Break(SourceSpan),
    /// `continue`
    Continue(SourceSpan),
    /// `return [expr]`
    Return(Option<HirExpr>, SourceSpan),
    /// `fail(expr)`
    Fail(HirExpr, SourceSpan),
    /// `attempt ... failed [err] ... end`
    Attempt(HirAttemptStmt),
    /// Local function declaration: `fn name(params) ... end`.
    ///
    /// Canon makes the binding exist when execution reaches the declaration, keeps the name
    /// visible inside the function's own body for recursion, and captures enclosing lexical
    /// bindings automatically.
    FnDecl(HirFunctionDecl),
    /// Expression statement.
    Expr(HirExpr),
}

/// HIR if statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirIfStmt {
    /// Main condition.
    pub condition: HirExpr,
    /// Main body.
    pub then_branch: Vec<HirStmt>,
    /// Elif branches.
    pub elif_branches: Vec<(HirExpr, Vec<HirStmt>)>,
    /// Else branch.
    pub else_branch: Option<Vec<HirStmt>>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR match statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirMatchStmt {
    /// Value being matched.
    pub target: HirExpr,
    /// Pattern arms.
    pub when_arms: Vec<(Vec<HirExpr>, Vec<HirStmt>)>,
    /// Fallback arm.
    pub else_arm: Option<Vec<HirStmt>>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR attempt-failed statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirAttemptStmt {
    /// Protected body.
    pub body: Vec<HirStmt>,
    /// Optional error binding name.
    pub error_binding: Option<String>,
    /// Fallback handler body.
    pub handler: Vec<HirStmt>,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HirExpr {
    /// Literal constant.
    Literal(Literal, SourceSpan),
    /// Identifier reference.
    Identifier(String, SourceSpan),
    /// Unary prefix operator.
    Unary(UnaryOp, Box<HirExpr>, SourceSpan),
    /// Binary operator (arithmetic, comparison, logical, range).
    Binary(BinaryOp, Box<HirExpr>, Box<HirExpr>, SourceSpan),
    /// Function call: `callee(args)`.
    Call(Box<HirExpr>, Vec<HirCallArg>, SourceSpan),
    /// Member dot access: `target.member`.
    Dot(Box<HirExpr>, String, SourceSpan),
    /// Safe navigation: `target?.member`.
    QuestionDot(Box<HirExpr>, String, SourceSpan),
    /// Index access: `target[index]`.
    Index(Box<HirExpr>, Box<HirExpr>, SourceSpan),
    /// List literal: `[a, b, c]`.
    List(Vec<HirExpr>, SourceSpan),
    /// Dictionary literal: `{k1: v1, k2: v2}`.
    Dict(Vec<(HirExpr, HirExpr)>, SourceSpan),
    /// Struct instantiation: `Type{x = 1, y = 2}`.
    Construct(String, Vec<(Option<String>, HirExpr)>, SourceSpan),
    /// Anonymous function closure: `fn(params) body end`.
    Fn(HirFunctionExpr),
    /// Conditional value expression: `if c then a else b`.
    If(Box<HirExpr>, Box<HirExpr>, Box<HirExpr>, SourceSpan),
    /// Failure fallback: `expr or_else fallback`.
    OrElse(Box<HirExpr>, Box<HirExpr>, SourceSpan),
}

/// Argument in an HIR call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirCallArg {
    /// Optional argument name.
    pub name: Option<String>,
    /// Argument value expression.
    pub value: HirExpr,
    /// Source span.
    pub span: SourceSpan,
}

/// HIR anonymous function expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HirFunctionExpr {
    /// Parameters.
    pub params: Vec<HirParam>,
    /// Optional return contract.
    pub return_type: Option<TypeAnnotation>,
    /// Function body statements.
    pub body: Vec<HirStmt>,
    /// Source span.
    pub span: SourceSpan,
}

impl HirExpr {
    /// Returns the source span for this HIR expression.
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            HirExpr::Literal(_, span)
            | HirExpr::Identifier(_, span)
            | HirExpr::Unary(_, _, span)
            | HirExpr::Binary(_, _, _, span)
            | HirExpr::Call(_, _, span)
            | HirExpr::Dot(_, _, span)
            | HirExpr::QuestionDot(_, _, span)
            | HirExpr::Index(_, _, span)
            | HirExpr::List(_, span)
            | HirExpr::Dict(_, span)
            | HirExpr::Construct(_, _, span)
            | HirExpr::If(_, _, _, span)
            | HirExpr::OrElse(_, _, span) => *span,
            HirExpr::Fn(f) => f.span,
        }
    }
}
