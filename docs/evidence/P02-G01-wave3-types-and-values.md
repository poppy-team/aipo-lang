# Evidência — P02-G01 / Wave 3 Tipos e Valores (Set, Sequence, Bytes Packing, Task, Duration)

**Goal:** `P02-G01` — Implementar os tipos/valores da Wave 3 (Set com ordem de inserção, Sequence lazy, packing de Bytes little-endian, Task como valor, Duration) com tags, prelude, display/igualdade e testes, sem mudar semântica existente.
**Fase:** P02 (Wave 3 — Async & Tipos Expandidos) · **Registrado em:** 2026-09-20
**Ambiente:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                           # 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0
$ cargo test --workspace                                               # 100% passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 (0 warnings)
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-js --test differential                            # 3 passed (VM == JS == stdout)
$ prumo validate .                                                     # clean
$ prumo doctor .                                                       # clean
```

## Resumo da Implementação

| Área | Decisões & Detalhes de Implementação |
|---|---|
| **Set** | `Value::Set(Rc<RefCell<Vec<Value>>>)` com ordem de inserção preservada, igualdade teórica de conjuntos (`==`), formatação `Set([a, b, ...])`. Métodos: `has(val)`, `add(val)`, `remove(val)`, `clear()`, `is_empty()`, `len()`, `to_list()`, `lazy()`. A chamada de conversão `Set(list)` remove duplicatas na ordem da primeira ocorrência. Espelho exato em `aipo-runtime.js` (`vSet`, métodos). |
| **Sequence** | `Value::Sequence(SequencePipeline)` representa uma cadeia de iteradores lazy imutável, iniciada a partir de uma List, de um Dict (valores) ou de um Set. Operações lazy de pipeline: `map`, `filter`, `flat_map`, `take`, `skip`, `distinct`, `chain`, `enumerate`. Consumidores terminais: `collect` (retorna `List`), `find`, `any`, `all`, `count`, `reduce`, `group_by`. A indexação drena e avalia os elementos. Espelhado em `aipo-runtime.js`. |
| **Bytes Packing** | `Value::Bytes` refatorado para `Rc<RefCell<Vec<u8>>>` para suportar mutação in-place e escritas em índice exato. Implementados métodos de packing binário com receiver primeiro e ordem little-endian: `read_i8`, `read_u8`, `read_i16`, `read_u16`, `read_i32`, `read_u32`, `read_i64`, `read_u64`, `read_f32`, `read_f64`, e os métodos `write_*` correspondentes. Conversão UTF-8: `String.encode()` -> `Bytes`, `Bytes.decode()` -> `String` (com `Failure` recuperável em UTF-8 inválido). O acesso fora dos limites gera fault de forma determinística. Paridade no backend JS usando `DataView` com a flag little-endian (`true`). |
| **Duration** | `Value::Duration(f64)` representando segundos com precisão em nível de microssegundo. Suporta aritmética (`+`, `-`, `-` unário), comparação relacional (`<`, `<=`), igualdade e o método `.total_seconds()`. A conversão `Duration(seconds)` aceita `Int` ou `Float`. Espelho exato no runtime JS (`vDuration`). |
| **Task & Group** | `Value::Task(TaskId)` e `Value::Group(GroupId)` representando handles de tasks assíncronas. Registrados como tipos de contrato built-in em `aipo-sema` e como `TypeTag` em `aipo-vm`, sem poluir o namespace global de valores como construtores (evitando colisões com structs de usuário chamadas `Task`, conforme verificado pela conformidade em `programs/10_integrated.aipo`). O módulo `task` fornece combinadores built-in: `spawn`, `sleep`, `all`, `race`, `timeout`, `cancel`, `group`. |
| **Paridade do Backend JS** | `crates/aipo-js/runtime/aipo-runtime.js` espelha 100% dos tipos, métodos, correspondências de tag, conversões de tipo e dicionários de módulos. Validado via `crates/aipo-js/runtime/selftest.mjs` e `crates/aipo-js/tests/differential.rs`. |

## Inventário de Testes & Verificação

- `crates/aipo-stdlib/tests/wave3_types.rs`: testes unitários cobrindo mutação e deduplicação de Set, cadeias lazy de Sequence, packing binário little-endian em todas as larguras numéricas (`i8`..`f64`), encode/decode UTF-8 de String e aritmética de Duration.
- `crates/aipo-vm/tests/wave3_pipeline.rs`: testes de integração end-to-end do pipeline da VM, compilando código-fonte Aipo com construções da Wave 3 diretamente para IR, bytecode e execução.
- `crates/aipo-js/tests/differential.rs`: testes diferenciais que verificam que programas executando tipos e operações da Wave 3 produzem stdout e exit codes idênticos nos runners da VM e do Node.js.
