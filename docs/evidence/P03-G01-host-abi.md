# Evidência — P03-G01 / Host ABI: Modelo de Capabilities, Valores do Host e Handles Geracionais

**Goal:** `P03-G01` — Provar a ABI neutra em relação ao host definida pela Wave 4: consumir uma AHS (descrição da superfície do host como dado), um modelo de capabilities deny-by-default, valores do host que são copiados por valor com identidade externa pertencente ao host, e handles geracionais que nunca fazem use-after-free (o acesso obsoleto resulta em none ou em uma Failure, conforme o contrato).
**Fase:** P03 (Wave 4 — Host ABI + Poppy) · **Registrado em:** 2026-09-21
**Ambiente:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                # 0 warnings
$ cargo test --workspace --all-targets                                 # 377 passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-cli --test conformance                            # programs 1–28, diagnostics 1–29
$ cargo test -p aipo-vm --test host_scope_escape                       # 9 passed (6 opcode sites + scope boundary)
$ cargo test -p aipo-vm --test host_capability_and_handles             # 5 passed (capability + stale handle)
$ cargo test -p aipo-host                                              # 37 unit + 1 doctest
```

## Verificação dos Critérios de Aceitação

| # | Critério | Status | Evidência |
|---|---|---|---|
| 1 | O modelo de capabilities é deny-by-default com a hierarquia do canon | ✅ | `CANON_CAPABILITIES` em `aipo-host/src/capability.rs`; `CapabilitySet::none()` começa vazio; 8 testes em `capability.rs` cobrem subsunção, estreitamento, fault de negação e boa formação da hierarquia do canon |
| 2 | A AHS é consumida como dado | ✅ | `HostSchema` em `aipo-host/src/ahs.rs` com `from_json`, `validate`, `declared_capabilities`, `missing_capabilities`, `require_capabilities`, busca por `function`; 9 testes cobrem superfícies bem formadas, duplicatas, subjects, async, capabilities malformadas, host vazio, JSON inválido |
| 3 | Valores do host são copiados por valor; nenhuma referência Rust cruza a fronteira | ✅ | `HostValue` em `aipo-host/src/value.rs` é um enum fechado de dados simples + `Handle`; `host_value_to_value`/`value_to_host_value` em `aipo-vm/src/host.rs` copiam cada variante por valor com normalização NFC na fronteira; valores mais ricos (coleções, funções) retornam `None` em `value_to_host_value`; 6 testes unitários |
| 4 | Handles geracionais nunca usam estado liberado | ✅ | `HandleTable` em `aipo-host/src/handle.rs` incrementa a geração em `remove`; a exaustão aposenta o slot em vez de dar wrap; `resolve` via `HostContext` retorna `VmFault::StaleHandle`; 10 testes unitários + 2 testes de integração de pipeline da VM (`test_stale_handle_via_host_context`, `test_stale_handle_across_slot_reuse`) |
| 5 | Capability ausente → AIPO_RT_CAPABILITY_DENIED com contexto de alvo/capability | ✅ | `CapabilitySet::require` → `HostFault::CapabilityDenied` → `VmFault::CapabilityDenied`; o teste de pipeline `test_capability_denied` com `revoke_clock()` prova que o fault dispara em runtime; `time.now`/`time.monotonic` testam o caminho positivo com `install_clock` |
| 6 | Um binding com escopo não pode escapar, imposto em 6 pontos de publicação no heap | ✅ | `Vm::publish_check` chamado em `SetGlobal` (L132), `Return` (L263), `SetField` (L381), `SetIndex` (L546), `BuildList` (L604), `BuildDict` (L619-620) em `dispatch.rs`; `HostContext::ensure_publishable` percorre containers recursivamente com um conjunto de visitados; 8 testes em nível de pipeline em `host_scope_escape.rs` cobrindo os 6 pontos |
| 7 | Fontes não determinísticas passam pela camada de capabilities, substituíveis por um profile de teste | ✅ | `time.now`/`time.monotonic` controlados pelo trait `ClockSource`; `install_clock`/`revoke_clock` globais ao processo; a CLI instala `SystemClock`; os testes instalam `FixedClock`; a negação gera fault com `AIPO_RT_CAPABILITY_DENIED`; programa de conformidade 28 |
| 8 | fmt, clippy, test, doc todos verdes | ✅ | Veja os gates acima |

## Resumo da Implementação

| Área | Detalhes |
|---|---|
| **Crate `aipo-host`** | `#![forbid(unsafe_code)]`. Cinco módulos: `ahs` (AHS como dado), `capability` (hierarquia deny-by-default), `fault` (`HostFault` de 5 variantes → códigos estáveis), `handle` (`HandleTable` geracional), `value` (enum fechado `HostValue` com verificações de fronteira). Depende apenas de `aipo-diagnostics` e `serde`. 37 testes unitários + 1 doctest. |
| **Adapter da VM (`aipo-vm/src/host.rs`)** | `HostContext` mantém `CapabilitySet`, `HandleTable<HostValue>`, escopos abertos e handles escapados. Conversões em ponto único: `host_value_to_value` (NFC, verificações de faixa), `value_to_host_value` (apenas dados simples), `host_fault_to_vm_fault` (mapeamento total, nunca um panic nem uma `Failure` recuperável). 14 testes unitários cobrindo round-trips, faults de fora de faixa, normalização NFC, recusa de referências, negação de capability, handles obsoletos, escape com escopo através de containers e structs, término do percurso autorreferencial e mapeamento completo de faults. |
| **Imposição de scope-escape** | `publish_check` em 6 pontos de publicação do bytecode protege valores prestes a se tornarem alcançáveis no heap. A saída antecipada em `!has_escapes()` (um teste de `bool`) faz com que programas sem bindings do host paguem praticamente nada. O percurso trata `Value::List`, `Value::Set`, `Value::Dict`, `Value::Struct` com um conjunto de visitados para parar em valores autorreferenciais. |
| **Módulo `time`** | `time.now()` → relógio de parede como `Duration`, `time.monotonic()` → relógio monotônico como `Duration`. Trait `ClockSource` com `install_clock`/`revoke_clock` para troca global ao processo. `SystemClock` usa `std::time::{SystemTime, Instant}`. A negação é um fault, nunca uma ausência silenciosa. O backend JS espelha o mesmo gate via `globalThis.__aipoClock`. |
| **Códigos de diagnóstico** | `AIPO_RT_CAPABILITY_DENIED`, `AIPO_RT_STALE_HANDLE`, `AIPO_RT_SCOPE_ESCAPE` em `DiagnosticCode` com `Severity::Fault`. Documentados em `docs/diagnostics/catalog.md`. |

## Inventário de Testes & Verificação

Testes unitários e de integração:

- `crates/aipo-host/src/` — 37 testes unitários + 1 doctest: subsunção e estreitamento de capabilities, deny-by-default, insert/get/release/obsoleto/exaustão de geração de handles, verificação de faixa de valores, validação de AHS.
- `crates/aipo-vm/src/host.rs` — 14 testes unitários: round-trips de valores, faults de fora de faixa, normalização NFC, recusa de referências, negação e concessão de capability, handles obsoletos, escape de escopo através de containers e structs, percurso autorreferencial, mapeamento de faults.
- `crates/aipo-vm/tests/host_scope_escape.rs` — 9 testes de integração que conduzem o pipeline real: pontos de escape SetGlobal, Return, BuildList, BuildDict, SetIndex, SetField; passagem por escopo aberto; fast path sem bindings do host; reuso de slot após o fechamento do escopo.
- `crates/aipo-vm/tests/host_capability_and_handles.rs` — 5 testes de integração: capability negada (revoke_clock → faults em time.now/monotonic), capability concedida (install_clock → sucesso), handle obsoleto via release, handle obsoleto através de reuso de slot, resolução de handle vivo.
- `crates/aipo-stdlib/src/time.rs` — 6 testes unitários: relógio negado/concedido, fontes fixas/com tick, leitura do relógio de parede.

Corpus de conformidade:

- `docs/conformance/programs/28_time_clock_capability.aipo` — monotônico ≥ 0, parede > 0, monotônico não decrescente.

## Não-Objetivos (Adiamentos Explícitos)

| Item | Justificativa |
|---|---|
| Escopos ECS, command buffer, behaviors/events, game.random | `aipo-poppy` (P03-G02) |
| Qualquer engine específica dentro de `aipo-host` | Permanece apenas com abstrações gerais |
| módulos de stdlib de filesystem/network/process | Stdlib de capabilities da Wave 6 |
| Imposição de orçamento de instruções/fuel, heap e tempo de parede | ADP-003 (indecidido) |
