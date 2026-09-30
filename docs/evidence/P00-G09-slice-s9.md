# Evidência — P00-G09 / Slice S9 (Runtime & Stdlib Mínima)

**Goal:** `P00-G09` — Runtime & Stdlib Mínima: registro de módulos, ordem de inicialização, Prelude V1 e módulos nativos da stdlib
**Fase:** P00 (Foundation) · **Slice:** S9 · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Gate: `fmt`

```
$ cargo fmt --all -- --check
(no output, exit 0)
```

## Gate: `check`

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Gate: `clippy` (os lints do workspace tratam warnings como erro)

```
$ cargo clippy --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)
(no warnings; workspace lints set `all/correctness/suspicious/complexity/perf/style = deny`)
```

## Gate: `test`

```
$ cargo test --workspace
total_passed=109 total_failed=0
```

Por crate:

| Crate | Testes |
|---|---|
| `aipo-stdlib` | 30 passaram, 0 falharam |
| `aipo-runtime` | 10 passaram, 0 falharam |
| `aipo-vm` | 24 passaram, 0 falharam |

Cobertura de `aipo-stdlib` (`crates/aipo-stdlib/tests/stdlib_tests.rs`, 30 testes):
Prelude V1 (`len`, `copy`, `same`, `some`, `fail`), conversões de tipos core
(`Int`, `Float`, `Byte`, `String`), `math` (`abs`, `min`, `max`, `floor`, `ceil`, `round`
half-away-from-zero, `truncate`, `sqrt`, `pow`, `clamp`), `string` (`len`, `byte_len`,
`contains`, `starts_with`, `ends_with`, `find`, `lower`, `upper`, `capitalize`, `reverse`,
`trim`, `split`, `join`, `replace`, `slice`, `format`), `io` (capture sink), regras de
argumentos do canon (padrões vazios de `split`/`replace`, elementos de `join` que não são
`String`, campos vazios preservados), cobertura da superfície de `NativeRegistry` e duas
execuções end-to-end da VM chamando nativos da stdlib via bytecode.

Cobertura de `aipo-runtime` (`crates/aipo-runtime/tests/graph_tests.rs`, 10 testes): grafo vazio,
módulo único, cadeia linear, desempate lexicográfico, dependência em diamante, dependência ausente,
ciclo direto, ciclo indireto, estados do ciclo de vida do módulo, operações do registro de nativos.

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings; `missing_docs` is enabled at workspace level in every crate)
```

## Gates de governança

```
$ prumo validate
Prumo validation passed.

$ prumo doctor
Prumo Doctor: all checks passed cleanly.

$ prumo docs verify
{"files":68,"checks":["authority","managed-regions","documentation-links","claim-drift","terminology","version-policy"],"findings":[],"ok":true}

$ prumo docs authority        # {"files":68,"canonical":68,"findings":[],"ok":true}
$ prumo docs contradictions   # []

$ prumo docs plan --goal P00-G09 --changed crates/aipo-stdlib docs/stdlib docs/adp docs/security --reconcile
{"predicted":["security.trust"],"actual":["security.trust"],"missing":[],"unpredicted":[],"drift_ratio":0,"findings":[]}

$ prumo goal list             # P00-G09 DONE, diagnostics [], warnings []
$ prumo report add            # TR-002 recorded in .prumo/history/project-intelligence.json
```

As obrigações de documentação previstas para os caminhos tocados eram `security.trust`; o delta
real foi o mesmo contrato, de modo que o reconcile de pós-execução (postflight) reporta drift zero.

## Aprovação de dependência

`unicode-normalization` 0.1.25 contém `unsafe` (`char::from_u32_unchecked`, decomposição de
Hangul). A aprovação foi solicitada e concedida explicitamente durante este slice; a
exceção está registrada em `docs/security/security-contract.md`. `unicode-segmentation`
1.13.3 (`#![deny(unsafe_code)]`) e `tinyvec` 1.13.3 (`#![forbid(unsafe_code)]`) foram
verificadas como livres de unsafe.

## Delta de documentação

| Artefato | Mudança |
|---|---|
| `docs/stdlib/mvp-subset.md` | novo — superfície implementada de Prelude V1 / `math` / `string` / `io`, modelo de erro, superfície adiada |
| `docs/adp/ADP-001-byte-and-core-types-as-values.md` | novo — questões em aberto: kind de runtime de `Byte`, valores de tipo de primeira classe, limites invertidos de `clamp`, saturação de `slice`, fronteiras NFC |
| `docs/crates/crate-contracts.md` | o contrato de `aipo-stdlib` agora aponta para a superfície implementada e para o ADP-001 |
| `docs/security/security-contract.md` | tabela de exceções de dependências aprovadas + requisito de varredura de unsafe |
| `docs/PRUMO.md` | o router aponta para contratos das crates, plano de waves, mapa de autoridade, subset da stdlib, ADP-001 |
| `crates/aipo-stdlib/README.md` | resumo da superfície, dos invariantes e do modelo de erro |
| `crates/aipo-runtime/README.md` | inalterado (preciso para este slice) |
| `PROJECT_STATE.md`, `CHANGELOG.md` | slice S9 registrado, próxima ação definida como S10 |
