# Aipo — Waves, Vertical Slices, Gauntlet Loops e Gates de Implementação

<aside>
⚙️

Esta página define como a Aipo sai de especificação para implementação sem big-bang. O mecanismo principal é vertical slice + evidence + gate; waves organizam dependências maiores; Gauntlet Loop aumenta qualidade sem mascarar falhas.

</aside>

## Vertical slice padrão

Toda feature percorre a menor cadeia observável aplicável:

```
spec/ADP
 → grammar
 → parser/recovery
 → AST/HIR
 → semantic analysis
 → Core IR
 → backend VM/JS
 → diagnostics
 → formatter/LSP
 → tests/conformance
 → docs
```

Não iniciar várias camadas incompletas quando um slice menor consegue provar arquitetura end-to-end.

## Wave model inicial

### Wave 0 — Engineering foundation

Workspace, CI, coding standards, diagnostics model, fixtures infrastructure, Prumo docs/contracts, xtask.

### Wave 1 — MVP executável da linguagem

A **primeira Wave do Gauntlet Loop não termina em uma fundação parcial**. Ela deve prosseguir continuamente, por vertical slices internos, até atingir o recorte formal de **MVP executável da Aipo**.

Subestágios internos da Wave 1 podem ser tratados como slices/goals separados, mas **não contam como encerramento da Wave**:

```
source model
 → lexer
 → lossless syntax + parser/recovery
 → AST/HIR
 → semantic baseline
 → Core IR
 → bytecode compiler
 → VM baseline
 → modules mínimos
 → stdlib mínima necessária
 → CLI run/check
 → diagnostics + formatter baseline
 → conformance/tests do MVP
```

O MVP da Wave 1 deve provar, end-to-end, pelo menos:

- arquivos `.aipo` carregados e executados pela CLI;
- literals e tipos fundamentais necessários ao recorte;
- `let`/`var`, bindings/scopes e mutabilidade básica;
- expressões, operadores e control flow mínimo aprovado;
- funções, chamadas, closures necessárias e retorno;
- `struct` e construção/uso básicos quando pertencentes ao MVP aprovado;
- List/Dict/Bytes ou o subconjunto mínimo explicitamente aprovado para o MVP;
- `none`, `Failure`/fault baseline e diagnostics estruturados;
- lowering Source → Syntax → HIR → Sema → Core IR → Bytecode → VM;
- módulos/imports mínimos suficientes para executar programas reais pequenos;
- stdlib mínima do MVP, sem tentar antecipar toda a V1;
- `aipo run` e `aipo check` funcionais;
- formatter baseline para a sintaxe implementada;
- fixtures pass/fail, snapshots e testes de integração end-to-end;
- nenhum `panic` Rust vazando como erro de usuário Aipo;
- documentação e contratos atualizados via Prumo.

O **Gauntlet Loop da Wave 1 continua iterando até esse gate de MVP estar satisfeito**. Não é permitido declarar a primeira Wave concluída apenas porque lexer, parser, semantic analysis ou VM isoladamente chegaram a alta qualidade.

### Wave 2 — JavaScript parity

Emitter, runtime shim, source maps, ESM, VM↔JS differential suite para o recorte já provado no MVP.

### Wave 3 — Async

Task, scheduler, await, await do, structured concurrency, cancellation, time/task.

### Wave 4 — Host ABI + Poppy

AHS, host values/handles, capabilities, ECS scopes/command buffer, behavior/events.

### Wave 5 — Tooling product

CLI ampliada, LSP completeness, test runner, docs, REPL, DAP/profiler baseline.

### Wave 6 — Packages + capability stdlib

Manifest/lock/resolver, expansão da stdlib, fs/http/net/crypto/process/env e sandbox policy.

### Wave 7 — Hot reload + hardening

Schema migration, deterministic replay, fuzz/security, performance baselines e proof programs finais.

Waves podem sobrepor apenas quando dependency/evidence DAG permitir. A numeração é planejamento inicial e pode ser refinada via Prumo sem quebrar contracts.

## Gate de entrada de feature

- contract/ADP suficiente;
- nenhuma contradiction bloqueante;
- dependencies implementadas ou simuladas por boundary estável;
- fixtures mínimas definidas;
- risks classificados;
- scope de crates conhecido.

## Gate de saída

- acceptance criteria verdes;
- pass/fail fixtures;
- diagnostics estáveis;
- snapshots atualizados;
- VM↔JS differential quando aplicável;
- formatter/LSP quando sintaxe/API pública mudou;
- security checks adequados ao risco;
- benchmark se hot path;
- docs delta resolvido;
- evidence registrada;
- nenhum TODO crítico oculto.

## Gauntlet Loop

Cada loop avalia dimensões reais como correctness, semantics, diagnostics, tests, safety, maintainability, modularity, portability, performance, documentation e agent-readiness.

Score 1–10 é derivado de rubrica e evidence. 10 significa cumprir o target documentado daquela wave, não software perfeito.

```
implement
 → validate
 → score by rubric
 → identify concrete gaps
 → fix highest-risk gaps
 → rerun deterministic checks
 → update evidence/docs
 → gate or repeat
```

## Stop conditions

Não refinar indefinidamente quando gates estão satisfeitos e gaps restantes são explicitamente deferred/non-blocking. Não alterar rubrica para aumentar score.

## Failure handling

Quando um slice falha arquiteturalmente, registrar experiment/rejection, preservar fixture que demonstrou o problema e voltar ao último gate estável. Não empilhar workaround sobre workaround.

## Quality budgets

Cada wave define budgets qualitativos/quantitativos apenas quando mensuráveis: diagnostic stability, parser recovery, memory/fuel, frame-time/GC debt, source-map correctness, package resolution determinism, compile/test duration etc.

## Proof programs finais

Release readiness exige pelo menos: jogo Poppy real; CLI/tool real; aplicação Web/JS real; package portátil VM↔JS. São integração de produto, não demos decorativas.