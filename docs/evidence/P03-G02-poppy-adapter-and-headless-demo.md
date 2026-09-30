# Evidência — P03-G02 / Adapter Poppy e Demo Headless Determinística

**Goal:** `P03-G02` — Implementar a crate `aipo-poppy` como o adapter da Poppy Game Engine sobre `aipo-host` (escopos ECS, command buffer, behaviors/events, simulação determinística) e prová-la com uma fixture de jogo de demonstração headless determinística.
**Fase:** P03 (Wave 4 — Host ABI + Poppy) · **Registrado em:** 2026-09-21
**Ambiente:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                # 0 warnings
$ cargo test --workspace --all-targets                                 # 391 passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-cli --test conformance                            # programs 1–28, diagnostics 1–29
$ cargo test -p aipo-poppy                                             # 14 passed (11 unit + 3 integration)
```

## Verificação dos Critérios de Aceitação

| # | Critério | Status | Evidência |
|---|---|---|---|
| 1 | Crate `aipo-poppy` com `#![forbid(unsafe_code)]` sobre `aipo-host` | ✅ | `crates/aipo-poppy/` criada, `#![forbid(unsafe_code)]`, importa `aipo-host`, camada limpa no workspace |
| 2 | Superfície do host Poppy descrita como dado via AHS (`HostSchema`) sob a árvore de capabilities `poppy` | ✅ | `poppy_schema()` em `schema.rs` expõe tipos (`Vec2`, `Transform`), o handle `Entity`, funções (`spawn`, `despawn`, `query`, `get_position`, `set_position`, `get_velocity`, `set_velocity`, `random_float`, `random_int`, `step`, `digest`), exigindo `poppy.ecs` e `poppy.random` |
| 3 | As identidades de entidades usam `Handle`s geracionais de `aipo-host` com detecção de handle obsoleto | ✅ | Entidades armazenadas em `HandleTable<EntityRecord>`; o acesso após o safe point a entidades removidas gera fault com `AIPO_RT_STALE_HANDLE` |
| 4 | Mutações estruturais do ECS registradas no command buffer, adiadas até os safe points | ✅ | `CommandBuffer` enfileira `Spawn`, `Despawn`, `SetPosition`, `SetVelocity`; aplicados em `World::apply_deferred()` durante `step()` |
| 5 | O ciclo de vida de behaviors segue a semântica de interface simples, sem keywords na linguagem | ✅ | A lógica de behavior no script do jogo opera sobre funções/métodos comuns, inspecionando entidades e emitindo comandos |
| 6 | A simulação Poppy headless é completamente determinística (tick fixo, PRNG com seed, ordenação do command buffer, digests idênticos entre execuções) | ✅ | `World::digest()` produz um digest de estado FNV-1a de 64 bits sobre slots de entidades ordenados, componentes, tick e estado do RNG; os testes verificam a igualdade bit a bit dos digests entre execuções |
| 7 | Fixture de demonstração headless executável verifica a simulação determinística de múltiplos ticks e a execução do command buffer | ✅ | `crates/aipo-poppy/tests/headless_demo.rs` compila e executa um script de jogo de múltiplos ticks, verificando a identidade de digests entre execuções com a mesma seed, a variação de digests com seeds diferentes, faults de handle obsoleto e a negação de capability |
| 8 | cargo fmt, clippy, test, doc todos verdes | ✅ | Veja os gates acima |

## Resumo da Implementação

| Componente | Responsabilidade |
|---|---|
| **`schema.rs`** | `poppy_schema()` canônico, que implementa o Aipo Host Schema (AHS) para o Poppy. Valida sem erros contra `HostSchema::validate()`. |
| **`prng.rs`** | `PoppyRng`: gerador de números pseudoaleatórios xorshift64* determinístico, com métodos de geração de float/int. |
| **`commands.rs`** | `CommandBuffer`: enfileira mutações estruturais (`Spawn`, `Despawn`, `SetPosition`, `SetVelocity`) para adiar modificações na tabela de entidades até os safe points. |
| **`world.rs`** | `World`: armazenamento de entidades apoiado por `HandleTable` geracional, filtro de query por tag, integração de velocidade e cálculo determinístico do digest de estado FNV-1a de 64 bits. |
| **`simulation.rs`** | `Simulation`: coordenador do game loop de taxa fixa, que avança ticks, física, flushes de comandos e o cálculo do digest. |
| **`adapter.rs`** | Integração com a VM: fornece o módulo `poppy` em `Vm.globals`, controlando as operações por trás das capabilities `poppy.ecs` e `poppy.random`, e traduzindo handles de entidades para `Value::HostHandle`. |
| **`tests/headless_demo.rs`** | Teste de integração end-to-end que conduz uma simulação de jogo de 10 ticks na VM do Aipo, verificando determinismo, detecção de handle obsoleto e imposição de capabilities. |

## Inventário de Testes & Verificação

Testes unitários e de integração:

- `crates/aipo-poppy/src/schema.rs` — validação limpa do schema, declaração de capabilities.
- `crates/aipo-poppy/src/prng.rs` — teste de determinismo, faixa de float `[0.0, 1.0)`.
- `crates/aipo-poppy/src/commands.rs` — enfileiramento e drenagem FIFO do command buffer.
- `crates/aipo-poppy/src/world.rs` — spawn de entidade, query por tag, despawn adiado, determinismo do digest.
- `crates/aipo-poppy/src/simulation.rs` — reprodutibilidade de simulação de 60 ticks entre instâncias.
- `crates/aipo-poppy/src/adapter.rs` — fault de negação de capability, execução concedida, fault de handle obsoleto no safe point.
- `crates/aipo-poppy/tests/headless_demo.rs` — execução do pipeline completo da VM com o script do jogo: digests idênticos com a seed 1337, digests diferentes com a seed 9999, fault de handle obsoleto após despawn, fault de negação de capability em ambiente sem concessão.
