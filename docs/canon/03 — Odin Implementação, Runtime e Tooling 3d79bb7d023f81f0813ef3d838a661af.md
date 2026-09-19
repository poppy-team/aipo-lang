# 03 — Odin: Implementação, Runtime e Tooling

<aside>
🔨

**Fonte primária de implementação.** Sempre que uma peça da Aipo puder ser estudada em código Odin oficial, preferir primeiro `core`/`vendor` e seus testes antes de adicionar dependência externa.

</aside>

## Linguagem e tooling básico

- [Odin Documentation](https://odin-lang.org/docs/)
- [Odin Overview](https://odin-lang.org/docs/overview/)
- [Running Tests](https://odin-lang.org/docs/testing/)
- [Odin Package Docs](https://pkg.odin-lang.org/)

**Uso:** sintaxe host, tipos, unions, allocators, context, testing e leitura da stdlib.

## Lexer/parser como material de estudo

- [core:odin/tokenizer](https://pkg.odin-lang.org/core/odin/tokenizer/) — tokenizer oficial para tooling.
- [core:odin/parser](https://pkg.odin-lang.org/core/odin/parser/) — parser oficial para tooling.
- [core:text/regex/parser](https://pkg.odin-lang.org/core/text/regex/parser/) — Pratt parser em Odin.
- [core:text/regex/compiler](https://pkg.odin-lang.org/core/text/regex/compiler/) — compiler para bytecode.
- [core:text/regex/virtual_machine](https://pkg.odin-lang.org/core/text/regex/virtual_machine/) — threaded VM de regex.

**Uso:** estudar padrões; não reutilizar semanticamente como frontend Aipo.

## Memória do compilador

- [core:mem](https://pkg.odin-lang.org/core/mem/) — allocators e infraestrutura de memória.
- [core:mem/virtual](https://pkg.odin-lang.org/core/mem/virtual/) — arenas com virtual memory.

**Política Aipo:** arenas para Source/AST/Semantic/Scratch; heap da linguagem completamente separado e gerido pelo tracing GC Aipo.

## Strings e interning

- [core:strings](https://pkg.odin-lang.org/core/strings/) — `Builder`, `Intern` e utilidades UTF-8.

**Uso:** symbol interning, emitters, diagnostics, disassembler e formatter.

## Grafo de módulos

- [core:container/topological_sort](https://pkg.odin-lang.org/core/container/topological_sort/)

**Uso:** ordenar módulos e diagnosticar ciclos de imports em `O(V+E)`.

## Handles seguros para host/game resources

- [core:container/handle_map](https://pkg.odin-lang.org/core/container/handle_map/)

**Uso:** textures, sounds, windows e resources externos expostos à Aipo via handles validados, nunca raw pointers.

## Async e concorrência futura

- [core:nbio](https://pkg.odin-lang.org/core/nbio/) — non-blocking I/O/event-loop infrastructure.
- [core:thread](https://pkg.odin-lang.org/core/thread/) — threads e thread pools.

**Política:** fibers pertencem à Aipo VM; `nbio` integra eventos do host. VM/heap permanece single-threaded inicialmente; workers não manipulam arbitrariamente objetos GC-managed.

## Games e multimedia

- [Odin vendor packages](https://pkg.odin-lang.org/vendor/)
- [vendor:raylib](https://pkg.odin-lang.org/vendor/raylib/)
- [vendor:sdl3](https://pkg.odin-lang.org/vendor/sdl3/)
- [vendor:wgpu](https://pkg.odin-lang.org/vendor/wgpu/)

**Baseline Game:** raylib primeiro para host mínimo; SDL3/wgpu somente quando existir caso de uso que justifique mais controle.

## Testes e memória

[Running Tests](https://odin-lang.org/docs/testing/) documenta test runner multithreaded e memory tracking, incluindo detecção de leaks/bad frees. Usar como camada externa de segurança do runtime Odin; isso não substitui testes do tracing GC da Aipo.

### Pipeline recomendado

```
odin check
→ odin test
→ golden tests
→ language conformance
→ --stress-gc
→ sanitizer/debug builds
→ VM ↔ JavaScript differential tests
→ benchmarks
```

## Regra de dependências

Antes de adicionar biblioteca externa, perguntar:

1. `core`/`vendor` já resolve?
2. implementar diretamente é menor e mais educativo?
3. a dependência entra no runtime distribuído ou apenas no tooling?
4. ela reduz mais complexidade do que introduz?

Dependências externas devem ser poucas e isoladas por adapter.