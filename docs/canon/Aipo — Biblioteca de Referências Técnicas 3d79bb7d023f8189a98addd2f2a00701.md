# Aipo — Biblioteca de Referências Técnicas

<aside>
📚

**Biblioteca viva de referência da Aipo.** Esta página organiza documentação oficial, papers, livros e implementações usados para orientar decisões de linguagem, compilador, VM, GC, backend JavaScript, Unicode, embedding e tooling. Uma fonte aqui não implica que a Aipo copiará seu design; cada item registra o problema para o qual ela serve de referência.

</aside>

## Como usar esta biblioteca

- **Primária:** fonte diretamente ligada ao baseline atual da Aipo.
- **Secundária:** comparação arquitetural ou sintática útil.
- **Pesquisa:** material para decisões futuras, sem compromisso de adoção.
- Priorizar documentação oficial, especificações, papers originais e código-fonte oficial.
- Antes de alterar semântica por influência externa, comparar com a Sintaxe Canônica e a Language Reference da Aipo.

## Índice

As subpáginas desta biblioteca serão separadas por domínio: linguagens de referência; compiladores/VM/GC; Odin e implementação; Web/Unicode/binário/tooling.

## Princípios de pesquisa

1. Usar a implementação mais simples compatível com a semântica desejada.
2. Medir antes de otimizar; otimizações de Lua/Luau/Wren são referências, não requisitos antecipados.
3. Manter a semântica Aipo independente da linguagem host e do backend JavaScript.
4. Tratar GC, FFI/host APIs e async como fronteiras arquiteturais explícitas.
5. Preservar fontes históricas relevantes mesmo quando a decisão atual segue outro caminho.

[01 — Linguagens de Referência e Design](01 — Linguagens de Referência e Design 3d79bb7d023f8154a455c05429f7bf1d.md)

[02 — Compiladores, Bytecode, VM e Garbage Collection](02 — Compiladores, Bytecode, VM e Garbage Collecti 3d79bb7d023f81e8b370ee07ac3edc00.md)

[03 — Odin: Implementação, Runtime e Tooling](03 — Odin Implementação, Runtime e Tooling 3d79bb7d023f81f0813ef3d838a661af.md)

[04 — Web, JavaScript, Unicode, Binário e Interoperabilidade](04 — Web, JavaScript, Unicode, Binário e Interoper 3d79bb7d023f813987a4c95197d90d96.md)

[05 — Referências Históricas de Decisão e Alternativas](05 — Referências Históricas de Decisão e Alternati 3d79bb7d023f81fb9ca3cd97efdf182f.md)

[06 — Testes, Fuzzing, Profiling e Sandboxing](06 — Testes, Fuzzing, Profiling e Sandboxing 3d79bb7d023f81e2941cefa0f7a8e151.md)