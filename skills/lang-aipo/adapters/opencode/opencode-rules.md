# Adaptador Aipo V1 para OpenCode CLI

Insira o conteúdo abaixo no seu `.opencoderules` ou nas instruções de projeto do OpenCode:

```markdown
# OpenCode Rules: Aipo V1 Language

- Target: Canonical Aipo V1 (ADR-001)
- Blocks: Braces `{}` only. 'end' is banned.
- Comments: `#` only. NEVER use `//` for comments (`//` is integer division).
- Iteration: `each x in xs { }` or `repeat N as i { }`. Banned: `for`.
- Methods: `TypeName:method_name(params) { }`. Banned: `class`, `impl`, `fn TypeName:method`.
- Batch Promotion: `TypeName::[fn1, fn2]` promotes free functions to methods. Functions MUST declare 'self' or 'var self' as 1st parameter.
- Structs: Data only. Field types (`field: Type`) are optional in grammar but STRONGLY RECOMMENDED.
- Enums: Supported closed sum type (`enum State { A, B(msg: String), C { x: Int }, }`). Exhaustive match without else.
- Mutability: Struct fields and local bindings are immutable by default. Mutable method receiver requires `var self`.
- Errors: `fail "reason"`, `attempt { } failed err { }`. Banned: `try`, `catch`, `finally`, `throw`, `defer`.
- Directives: Supported `#!satisfies`, `#!test`, `#!deprecated("msg")`, `#!todo("msg")`.
- Logic: `and`, `or`, `not`. Banned: `&&`, `||`, `!`.
- Semicolons: Banned `;`. Newlines terminate statements.
```
