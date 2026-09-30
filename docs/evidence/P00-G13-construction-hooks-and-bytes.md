# Evidência — P00-G13 / Hooks de Construção do MVP & `Bytes`

**Goal:** `P00-G13` — Fechamento de contratos do MVP: `init`, `invariant`, contratos em runtime e construção de `Bytes`
**Fase:** P00 (Foundation) · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Por que este goal existiu

O corpus de conformidade do S11 (`docs/evidence/P00-G12-conformance-and-mvp-gate.md`) encontrou quatro pontos
em que o canon e a implementação discordavam, e registrou o gate do MVP como **parcial** em vez de
fechá-lo. Este goal resolve três deles a partir do canon e re-delega o quarto.

| Lacuna | Fonte da decisão | Resultado |
|---|---|---|
| G1 — `init` não invocado por `Type{...}` | Sintaxe Canônica: `Type{...}` usa `init` quando ele existe; o hook não é uma dot-call | **fechada** |
| G2 — `invariant()` não avaliado | Sintaxe Canônica: verificado ao final da construção; linhas combinadas por `and` | **fechada (na construção)** |
| G4 — `Bytes` não tinha forma de construção | Language Reference §6: `let data = Bytes(32)` — um bloco mutável gerenciado | **fechada** |
| G2b — invariante nos pontos de mutação | Sintaxe Canônica: `candidate → apply → verify → commit` | adiada para `P00-G14` |
| G3 — contratos de assinatura em runtime | Plano da Wave 1: "verificação em runtime nas fronteiras" | adiada para `P00-G14` |

O canal de erro para ambas as violações de contrato já havia sido decidido pelo próprio modelo de erro do MVP do
repositório (`docs/stdlib/mvp-subset.md`): **violações de contrato são faults de runtime**, e não
`Failure`s recuperáveis. Nenhuma semântica nova precisou ser inventada.

## Implementação

| Estágio | Mudança |
|---|---|
| `aipo-syntax` | `parse_init_hook` injeta o receiver implícito `self!` quando o autor o omite, de modo que o `init(id, name)` do canon e a forma explícita `init(self!, id)` fazem lowering de forma idêntica |
| `aipo-hir` | `invariant()` faz lowering para um predicado que recebe `self`: as linhas do hook viram uma cadeia de `and` retornada por `<Type>.invariant` |
| `aipo-ir` | os hooks `init` são registrados como assinaturas de construção (apenas parâmetros declarados); `Type{...}` emite `BuildStruct(defer_fixed)` → chamada a `Type.init` → `Pop` → verificação do invariante → `SealStruct`; `init` retorna a instância que recebeu |
| `aipo-bytecode` | `BuildStruct` carrega uma flag `defer_fixed`; novos opcodes `SealStruct` e `AssertInvariant` (emitter, verifier com verificações de limites do índice de nomes, disassembler) |
| `aipo-vm` | `StructInstance::under_construction` mantém os campos `fixed` mutáveis até o `seal`; `BuildStruct` adia tanto o conjunto `fixed` quanto o invariante; `SealStruct` restaura o conjunto; `AssertInvariant` transforma um predicado `false` em `VmFault::InvariantViolation`; conversão `Bytes(count)` |

## Gate: `fmt` / `check` / `clippy`

```
$ cargo fmt --all -- --check
FMT=0

$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo clippy --workspace --all-targets -- -D warnings
(zero warnings)
```

## Gate: `test`

```
$ cargo test --workspace
passed: 137  failed: 0
```

Nova certificação:

| Fixture | Comprova |
|---|---|
| `docs/conformance/programs/13_init_and_invariant.aipo` | `init` do canon com defaults, atribuição a `fixed` durante a construção, default que referencia um parâmetro anterior, invariante verificado ao final da construção |
| `docs/conformance/diagnostics/11_runtime_invariant_violation.aipo` | um invariante violado falha com `AIPO_RT_TYPE_MISMATCH` e a instância nunca é publicada |
| `docs/conformance/programs/12_bytes.aipo` | tamanho de `Bytes(4)`, indexação de bytes, `len`, slicing, `Bytes(0)` |
| `crates/aipo-stdlib/tests/stdlib_tests.rs::test_convert_bytes` | forma de `Bytes(count)`, preenchimento com zeros, `Bytes(0)`, `Failure` para negativo/grande demais, fault para não `Int` |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G1, G2 e G4 marcadas como resolvidas, com
  a decisão de implementação registrada; G2b e G3 reescritas como critérios do próximo goal.
- `docs/stdlib/mvp-subset.md` — a seção de `Bytes` agora documenta a forma canônica `Bytes(count)`
  e o limite provisório de alocação.
- `docs/conformance/README.md` — matriz de snapshots estendida; a seção "Lacunas verificadas" agora declara
  quais lacunas foram fechadas e quais permanecem.
- `CHANGELOG.md`, `PROJECT_STATE.md`.

## Limitações conhecidas (registradas, não escondidas)

- `invariant()` é verificado apenas ao final da construção. Mutar um campo após a publicação
  ainda não reavalia o predicado — lacuna **G2b**, atribuída ao `P00-G14`.
- Corpos de `init` que executam um `return` explícito deixam como valor da construção esse return
  em vez da instância. O `init` do canon é um hook sem valor, então isso só é alcançável ao
  escrever um corpo incomum; o caminho normal retorna a instância selada.
- `BYTES_MAX_ALLOCATION` (64 MiB) é uma guarda provisória da implementação, não um limite do canon.
