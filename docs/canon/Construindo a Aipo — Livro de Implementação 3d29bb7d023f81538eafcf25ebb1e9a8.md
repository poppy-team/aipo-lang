# Construindo a Aipo — Livro de Implementação

<aside>
🦀

**Atualização arquitetural — 2026-09-15:** o livro passa a acompanhar a implementação **Rust-first e code-agent-first**. As seções históricas em Odin continuam úteis como registro de decisões anteriores, mas não definem mais a baseline vigente. A arquitetura atual, o Poppy Profile, o backend JS e o roadmap estão consolidados em [Aipo — Rust/Poppy Pivot, Host Profiles e Roadmap 10/10](Aipo — Rust Poppy Pivot, Host Profiles e Roadmap 1 3dc9bb7d023f81a4b03fe8f7e6de3508.md).

</aside>

<aside>
📚

**Objetivo:** transformar toda a implementação da Aipo em um livro vivo, incremental e reutilizável para revisão futura ou ensino de outras pessoas.

</aside>

## Regra editorial permanente

- Cada fase concluída deve ser registrada.
- Cada conceito importante aprendido deve ser documentado com explicação, exemplo e relação com a Aipo.
- Decisões arquiteturais devem apontar para o problema, alternativas consideradas, decisão e razão.
- Exercícios, erros relevantes e descobertas que ajudem o aprendizado podem virar exemplos ou notas de capítulo.
- Quando uma página crescer demais ou misturar assuntos diferentes, o conteúdo deve ser dividido em subpáginas.
- O livro deve manter ordem pedagógica, não apenas ordem cronológica.

## Estrutura planejada

1. Como estudar e construir a Aipo
2. Parte I — Source, Lexer e Tokens
3. Parte II — Parser e AST
4. Parte III — Análise semântica
5. Parte IV — Representações intermediárias e bytecode
6. Parte V — Máquina virtual
7. Parte VI — Runtime, objetos e memória
8. Parte VII — Garbage Collector
9. Parte VIII — Módulos, pacotes e toolchain
10. Parte IX — Embedding, backend JavaScript, DSLs e otimização
11. Apêndices — Rust, ferramentas, decisões, experimentos e baseline Odin histórica

## Convenção de cada capítulo

Sempre que fizer sentido, um capítulo seguirá a sequência: **problema → conceito → exemplo → implementação → teste → explicação → decisões → revisão**.

## Arquitetura de implementação vigente

```
.aipo source
    ↓
Source + Lexer
    ↓
Parser + AST
    ↓
Semantic analysis
    ↓
Core IR
    ├──→ Bytecode emitter → .aibc → Aipo VM
    └──→ JavaScript emitter → Web/JS host

Aipo VM (Rust)
    ├── Value model
    ├── Stack / frames / fibers
    ├── Runtime / stdlib
    └── Aipo Heap → tracing GC
```

### Restrições deliberadas

- Não implementar backend C/native da linguagem no roadmap atual.
- Não introduzir LLVM/QBE/JIT antes de existir necessidade medida.
- Não transformar RC/ownership inference em requisito do runtime.
- Parser explícito + bytecode VM + tracing GC managed em Rust são o caminho de referência. Rowan/lossless syntax, snapshots, fuzzing e suíte de conformidade passam a ter prioridade por melhorarem tooling e desenvolvimento com code agents.
- DSLs de HTML/CSS/UI/scene/config reutilizam trailing blocks e builders em biblioteca.
- Async/await é fase posterior ao baseline da VM e deve baixar para fibers/event loop no runtime principal.

## Relação com outros documentos

A especificação viva continua sendo a fonte normativa de **como a linguagem Aipo se comporta**. Este livro registra **como e por que ela foi construída**, incluindo Rust, a baseline Odin histórica, frontend, compilador, Core IR, bytecode, VM, runtime, tracing GC, backend JavaScript, embedding/Poppy Profile, tooling e desenvolvimento code-agent-first.

[Como estudar este livro — Método de Aprendizagem](Como estudar este livro — Método de Aprendizagem 3d29bb7d023f8196bc1fc80efb3281d3.md)

[Parte I — Source, Lexer e Tokens](Parte I — Source, Lexer e Tokens 3d29bb7d023f8156a0a2d3f63cf4f4fa.md)

## Referência normativa de sintaxe

Durante a implementação, exemplos do livro devem usar a superfície consolidada da Aipo V1. Quando material pedagógico antigo divergir, deve ser revisado ou explicitamente marcado como histórico.

[Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md)

A revisão de 2026-09-07 introduziu como formas canônicas `fixed` + `invariant`, `each`, `impl` + `self`, `match/when`, `attempt/failed` e a distinção entre função sem valor e `none`.

[Interlúdio — Revisão da Sintaxe antes da Implementação](Interlúdio — Revisão da Sintaxe antes da Implement 3d59bb7d023f817680b6d9735250227d.md)

[Interlúdio — Fixed, Visibilidade, Mutabilidade e Escopo](Interlúdio — Fixed, Visibilidade, Mutabilidade e E 3d59bb7d023f81c4bad6d1c418654e54.md)