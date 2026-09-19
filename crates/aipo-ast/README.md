# aipo-ast

`aipo-ast` defines strongly typed Abstract Syntax Tree (AST) nodes, items, declarations, statements, expressions, and operators for Aipo V1.

## Architecture & Guarantees
- **Lossless Spans**: Every AST node retains its full source boundary via `SourceSpan`.
- **Typed Item & Stmt Hierarchy**: Distinguishes structural module items (`Item::Fn`, `Item::Struct`, `Item::Impl`, `Item::Interface`, `Item::Satisfy`, `Item::Import`, `Item::Export`) from block statements (`Stmt::Let`, `Stmt::Var`, `Stmt::If`, `Stmt::Match`, `Stmt::Loop`, `Stmt::While`, `Stmt::Repeat`, `Stmt::Each`, `Stmt::Attempt`, etc.).
- **Serialization**: Derives `Serialize` and `Deserialize` for tooling, AST dumping, and snapshot verification.
- **Safety**: `#![forbid(unsafe_code)]`.
