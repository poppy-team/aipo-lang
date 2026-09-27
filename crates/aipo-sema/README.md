# aipo-sema

`aipo-sema` implements semantic analysis, lexical scope resolution, mutability checking, and contract verification for the Aipo compiler.

## Architecture & Guarantees
- **Scope Hierarchy**: Arena-backed lexical scope tree tracking bindings across modules, functions, and nested blocks.
- **Redeclaration Rules**: Prohibits redeclaring bindings within the same scope (`AIPO_SEM_REDECLARED_IN_SCOPE`) while cleanly supporting nested shadowing.
- **Mutability Verification**: Distinguishes `let` (immutable) from `var` (mutable), enforces immutable variables and struct fields by default, and prevents field mutation through immutable receivers without `var self`.
- **Contract & Arity Checking**: Validates static call arities against function definitions, and checks automatic structural interface satisfaction.
- **Safety**: `#![forbid(unsafe_code)]` with zero compiler panics.
