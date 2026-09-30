# Evidência — P02-G02 / Wave 3 Stdlib: Combinadores Assíncronos e Operações de Task

**Goal:** `P02-G02` — Implementar e certificar combinadores assíncronos (`task.spawn`, `task.sleep`, `task.all`, `task.race`, `task.timeout`, `task.cancel`, `task.group`) com tempo virtual determinístico e paridade VM↔JS.
**Fase:** P02 (Wave 3 — Async & Tipos Expandidos) · **Registrado em:** 2026-09-20
**Ambiente:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                           # 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0
$ cargo test --workspace                                               # 100% passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 (0 warnings)
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-js --test differential                            # 8 passed (VM == JS == stdout)
$ prumo validate .                                                     # clean
$ prumo doctor .                                                       # clean
```

## Resumo da Implementação

| Área | Decisões & Detalhes de Implementação |
|---|---|
| **Scheduler Determinístico** | Scheduler cooperativo de tasks, single-threaded e determinístico, implementado tanto na VM em Rust (`aipo-vm/src/vm/task.rs`) quanto no runtime JavaScript (`aipo-runtime.js`). As tasks executam em profundidade, com run queue FIFO. O tempo virtual (`tick`) avança estritamente quando tasks dormem ou aguardam deadlines — computação pura nunca avança o relógio. |
| **`task.spawn`** | `task.spawn(fn, args: List) -> Task`. Cria uma nova task inicializada com seu próprio frame, stack e ambiente de upvalues, enfileirando-a na `run_queue`. |
| **`await`** | Resolve tasks concluídas inline (fast path). Para tasks pendentes, suspende o waiter com a operand stack e o instruction pointer intactos, entregando a execução diretamente à task aguardada antes do restante da fila. A detecção de ciclos interrompe com `AIPO_RT_AWAIT_CYCLE`. A invocação dentro de callbacks síncronos do host gera fault com `AIPO_RT_AWAIT_IN_CALLBACK`. Aguardar tasks canceladas gera fault com `AIPO_RT_CANCELLED`. |
| **`task.sleep`** | `task.sleep(duration_or_ticks)`. Aceita ticks `Int`/`Byte` ou `Duration` (1s = 1000 ticks). Valores negativos produzem `Failure` recuperável. `sleep(0)` cede a vez para outras tasks enfileiradas como um ponto de reagendamento determinístico. Ao retomar, a reexecução da chamada verifica o tempo virtual e resolve com `None`. |
| **`task.all`** | `task.all(tasks: List) -> List`. Join estruturado que aguarda todos os membros. A primeira falha na ordem de conclusão faz o join falhar; caso contrário, os resultados são coletados na ordem original dos membros. Se algum membro for cancelado, gera fault com `AIPO_RT_CANCELLED`. Lista vazia retorna `[]`. |
| **`task.race`** | `task.race(tasks: List) -> Value`. A primeira conclusão de membro na ordem determinística do scheduler vence; vencedores cancelados geram fault. Lista vazia retorna `Failure("race of no tasks")`. |
| **`task.timeout`** | `task.timeout(task, deadline_or_duration) -> Value`. Resolve com o resultado da task se concluída antes do deadline; na expiração do deadline, marca o join como `Failure("timeout")` e cancela a task atrasada. Alvo vazio retorna `Failure("timeout of no task")`. |
| **`task.cancel`** | `task.cancel(task) -> None`. Marca a task como `Cancelled`. Conduzir ou aguardar uma task cancelada gera fault de forma determinística (`AIPO_RT_CANCELLED`). |
| **`task.group`** | `task.group() -> Group`. Cria um escopo de concorrência estruturada com `group.spawn(fn, args: List) -> Task` e `group.wait() -> List`. Spawns tardios se juntam dinamicamente aos waits de grupo abertos. Coleta os resultados dos membros na ordem de conclusão. |
| **Paridade de Backend VM ↔ JS** | `crates/aipo-js/runtime/aipo-runtime.js` contém um scheduler cooperativo 100% equivalente, com a máquina de estados exata (`Pending`, `Running`, `Sleeping`, `Blocked`, `Ready`, `Failed`, `Cancelled`), mecânica de join, polling de timeout e despacho do opcode `case 'Await':`. Avaliado e certificado via `selftest.mjs` e `crates/aipo-js/tests/differential.rs`. |

## Inventário de Testes & Verificação

- `crates/aipo-vm/tests/wave3_pipeline.rs`: testes de integração end-to-end cobrindo `spawn_and_await`, `all_and_race`, `group`, `sleep_and_timeout`, `timeout_expires` e `cancel`.
- `crates/aipo-js/runtime/selftest.mjs`: asserções unitárias para `vGroup`, igualdade e formatação de `Group`, além de execução assíncrona com `runModule`.
- `crates/aipo-js/tests/differential.rs`: suíte diferencial que compila programas assíncronos de Aipo para bytecode (para a VM) e para bundle de Core IR (para o Node.js), verificando igualdade exata de stdout e de status code em:
  - `test_wave3_async_spawn_await_differential`
  - `test_wave3_async_all_and_race_differential`
  - `test_wave3_async_group_differential`
  - `test_wave3_async_sleep_and_timeout_differential`
  - `test_wave3_async_cancel_differential`
