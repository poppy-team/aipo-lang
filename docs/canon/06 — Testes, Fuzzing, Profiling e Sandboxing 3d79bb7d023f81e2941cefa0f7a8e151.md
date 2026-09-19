# 06 — Testes, Fuzzing, Profiling e Sandboxing

<aside>
🧪

**Objetivo:** tornar correção, segurança e performance propriedades verificáveis da implementação, especialmente porque lexer/parser, bytecode, VM e GC processam entradas potencialmente hostis ou malformadas.

</aside>

## Test runner e memória do host

- [Odin — Running Tests](https://odin-lang.org/docs/testing/)
- [Odin sanitizer package](https://pkg.odin-lang.org/base/sanitizer/)

**Uso:** `odin test`, memory tracking por teste, leak/bad-free detection e integração com AddressSanitizer/MemorySanitizer quando disponível.

## Estratégia de testes da Aipo

Manter suites separadas para:

- lexer golden tests;
- parser/AST golden tests;
- diagnostics golden tests;
- semantic/Core IR tests;
- bytecode/disassembly golden tests;
- VM execution/conformance;
- `--stress-gc` coletando agressivamente durante testes;
- differential tests VM ↔ JavaScript;
- malformed-input tests;
- performance regression benchmarks.

## Fuzzing

### AFL++

- [AFL++ Documentation](https://aflplus.plus/docs/)
- [AFL++ Quick Start](https://aflplus.plus/docs/quickstartguide/)

**Uso futuro:** fuzz externo de parser/compiler/bytecode loader quando houver harness estável e integração de build adequada.

### Fuzzer estrutural próprio

Antes de integrar ferramenta pesada, criar geradores Odin simples de:

- bytes arbitrários para lexer/UTF-8;
- tokens/expressões aleatórios;
- AST/programas pequenos válidos;
- bytecode inválido para validar loader/validator;
- grafos de objetos para stress do GC.

A propriedade mínima do frontend é: entrada arbitrária pode produzir programa válido ou diagnóstico, nunca memory corruption/crash não controlado.

## Sandboxing e código não confiável

### Luau sandbox

- [Embedding a sandboxed Luau VM](https://luau.org/sandbox/)
- [Luau C API — Sandboxing](https://luau.org/api/#sandboxing)

**Uso:** referência principal para uma futura Aipo embutida executando scripts não confiáveis.

Princípios a preservar:

- standard library não deve conceder acesso irrestrito ao sistema quando em sandbox;
- host capabilities devem ser explicitamente fornecidas pelo embedder;
- bytecode externo não deve ser tratado como seguro automaticamente;
- execução precisa de mecanismo de interrupção/budget para scripts que não terminam;
- memória deve poder ser limitada/monitorada por VM;
- isolamento forte pode exigir VMs separadas para trusted/untrusted code.

## Profiling

### Tracy Profiler

- [Tracy repository/manual](https://github.com/wolfpld/tracy)

**Uso futuro:** CPU zones, frame timing, allocations, locks e análise de GC/VM em workloads de games. Só integrar após benchmarks internos apontarem gargalos que timing simples não explica.

### Métricas próprias obrigatórias antes de profiler externo

A VM deve conseguir emitir estatísticas baratas:

```
instructions_executed
bytes_allocated
gc_cycles
gc_pause_total
gc_pause_max
live_heap_bytes
peak_heap_bytes
objects_allocated
function_calls
```

Essas métricas permitem comparar versões e identificar regressões sem acoplar o runtime a uma ferramenta externa.

## Tooling de editor

### Tree-sitter

- [Tree-sitter Introduction](https://tree-sitter.github.io/tree-sitter/)
- [Creating Parsers](https://tree-sitter.github.io/tree-sitter/creating-parsers/1-getting-started.html)

**Uso futuro:** syntax highlighting, structural navigation e tooling incremental. Não é o parser canônico do compilador da Aipo.

## Source-level debugging no backend JavaScript

- [ECMA-426 — Source Map Format](https://ecma-international.org/publications-and-standards/standards/ecma-426/)
- [Latest ECMA-426 draft](https://tc39.es/ecma426/)

**Uso:** gerar source maps do `.js` para `.aipo`, preservando localização original em stack traces e devtools.

## Regra de maturidade

Cada nova otimização de VM/GC deve entrar acompanhada de:

1. benchmark que demonstra o gargalo;
2. conformance tests antes/depois;
3. stress/fuzz relevante;
4. medição de memória/latência;
5. possibilidade de desativar ou comparar com implementação de referência quando viável.