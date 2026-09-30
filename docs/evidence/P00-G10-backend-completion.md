# Evidência — P00-G10 / Conclusão do Backend (lacunas dentro de S1–S9)

**Goal:** `P00-G10` — Conclusão do Backend: funções, closures, coleções, Byte/Range/slice e execução de módulos
**Fase:** P00 (Foundation) · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Por que este goal existiu

Os slices S1–S9 entregaram cada estágio do pipeline com seus próprios testes, mas os estágios ainda
não estavam ligados entre si de ponta a ponta: o parser produzia chamadas que o emitter de bytecode não
conseguia fazer lowering, a VM não tinha valores de função de primeira classe, a API de coleções parava nas operações
que não precisam de callback, e `import`/`export` foram especificados no S9, mas nunca executados. Este goal
fecha essas lacunas para que um programa `.aipo` possa usar todo o subset do MVP por meio de `aipo run`.

## O que mudou, por estágio

| Estágio | Crate | Lacuna fechada |
|---|---|---|
| Frontend | `aipo-syntax` | a interpolação `f"..."` sofre desugaring para concatenação de `String(...)`; `a[a..b]`, `a[..b]`, `a[a..]` e `a[..]` fazem parse como expressões de índice por range em vez de um simples `a[a]` |
| Core IR | `aipo-ir` | declarações de struct carregadas em `Program`; `BuildStruct` inicializa os campos em ordem canônica; dois novos opcodes para parâmetros default e para o prologue no lado do callee |
| Bytecode | `aipo-bytecode`, `aipo-bytecode::module` | tabela de structs em `BytecodeModule`; emitter, verifier e disassembler cobrem todos os novos opcodes |
| VM | `aipo-vm` | valores de função de primeira classe e closures com upvalues; `Value::Byte`/`Value::Bytes`/`Value::Type`; testes de tipo; indexação por range e slice; frames de chamada cientes do receiver; imposição de `MutationDuringIteration` na iteração de `List`/`Dict` |
| Stdlib | `aipo-stdlib` | `collections.rs` vincula os métodos de `List`/`Dict` à VM; as conversões delegam à VM para que `String(v)` e `io.print(v)` não possam divergir; superfície de métodos de `String`/`Bytes` |
| Sema | `aipo-sema` | a superfície do Prelude (`docs/…/mvp-subset.md`) fica visível para a resolução de nomes |
| CLI | `aipo-cli` | `modules.rs` resolve e executa `import`/`export` com ordem topológica e semântica de init-once |

## Gate: `fmt`

```
$ cargo fmt --all -- --check
FMT_EXIT=0
```

## Gate: `check`

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Gate: `clippy` (os lints do workspace tratam warnings como erro)

```
$ cargo clippy --workspace --all-targets -- -D warnings
(no output, exit 0)
```

## Gate: `test`

```
$ cargo test --workspace
passed: 136  failed: 0
```

Por crate:

| Crate | Testes | | Crate | Testes |
|---|---|---|---|---|
| `aipo-source` | 5 | | `aipo-ir` | 2 |
| `aipo-diagnostics` | 3 | | `aipo-bytecode` | 4 |
| `aipo-lexer` | 5 | | `aipo-vm` | 24 |
| `aipo-syntax` | 13 | | `aipo-runtime` | 10 |
| `aipo-ast` | 1 | | `aipo-stdlib` | 30 |
| `aipo-hir` | 4 | | `aipo-formatter` | 11 |
| `aipo-sema` | 8 | | `aipo-cli` | 16 |

O trabalho de fechamento das lacunas é fixado pelo corpus de conformidade, e não apenas por testes
unitários. Cada fixture de programa em `docs/conformance/programs/` é executada pelo pipeline real e comparada
com o stdout commitado, e cada fixture de diagnóstico precisa falhar com o código commitado:

| Fixture | Lacuna que comprova fechada |
|---|---|
| `02_recursion`, `11_defaults_and_named_args` | funções com locals/params reais, parâmetros default avaliados a cada chamada, defaults que referenciam parâmetros anteriores, argumentos nomeados em qualquer ordem, pipeline para uma chamada |
| `06_closures` | `fn` anônima, captura, captura por iteração em loops |
| `04_collections` | API de métodos de `List`/`Dict`, incluindo os de ordem superior `transform`, `filter` e `sort_by` (que chamam de volta o Aipo pela VM) |
| `09_slicing` | ranges `a..b`, slicing de list/string com limites negativos, limites de slice omitidos |
| `05_structs_and_impl` | `struct` com defaults, construção, métodos de `impl`, funções associadas |
| `07_strings_and_math` | `Byte(255)`, superfície de `string`/`math`, interpolação `f"..."` |
| `diagnostics/08_runtime_mutation_during_iteration` | `AIPO_RT_MUTATION_DURING_ITERATION` |
| `modules/basic`, `modules/cycle`, `modules/missing` | execução de import/export, init-once, privacidade, `AIPO_SEM_IMPORT_CYCLE`, `AIPO_SEM_UNKNOWN_MODULE` |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## Gate: `documentation_impact`

- `docs/stdlib/mvp-subset.md` — a execução de módulos deixa de constar como exclusiva do S9; a superfície de
  métodos de `List`/`Dict` e o motivo de os métodos de ordem superior viverem na VM estão registrados.
- `docs/adp/ADP-001-byte-and-core-types-as-values.md` — Q1 e Q2 são resolvidas por este slice
  (`Value::Byte` e valores de tipo de primeira classe agora existem); as demais questões em aberto continuam abertas.
- `docs/evidence/P00-G12-conformance-and-mvp-gate.md` — o corpus que este slice valida é pontuado
  ali.
- `CHANGELOG.md`, `PROJECT_STATE.md`.
