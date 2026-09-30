# Evidência — P00-G11 / Slice S10 (CLI + Formatter)

**Goal:** `P00-G11` — CLI e Formatter: `aipo run`, `check`, `fmt`
**Fase:** P00 (Foundation) · **Slice:** S10 · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Entregável

Duas crates fecham o pipeline executável. Trabalhando de trás para frente a partir da superfície de comandos em
`docs/reference/cli.md`:

| Crate | Responsabilidade |
|---|---|
| `aipo-formatter` | renderização determinística e idempotente de um arquivo-fonte a partir de seu stream de tokens |
| `aipo-cli` | parsing de argumentos, orquestração `source → syntax → HIR → sema → IR → bytecode → VM`, resolução de módulos, emissão de diagnósticos e exit codes |

O workspace agora contém as 14 crates que o plano da Wave 1 prevê
(`cargo metadata` lista todas as crates em `crates/`).

### Superfície de comandos

```
aipo run <path> [--message-format=<human|jsonl>]
aipo check <path> [--message-format=<human|jsonl>]
aipo fmt <paths...> [--check]
aipo --version · aipo --help
```

Os exit codes fazem parte do contrato e cada um é exercitado por um teste:

| Código | Significado |
|---|---|
| `0` | o programa executou até o fim, `check` não encontrou erro, ou `fmt` escreveu/verificou os arquivos |
| `1` | falha de linguagem: qualquer diagnóstico de severidade error, uma falha de verificação de bytecode, um fault de runtime não capturado, ou drift em `fmt --check` |
| `2` | erro de uso: comando ou flag desconhecido, argumento ausente, arquivo ilegível |

### Resolução de módulos

`crates/aipo-cli/src/modules.rs` implementa o modelo de módulos do S9 de ponta a ponta, que os
slices anteriores haviam apenas especificado:

- um arquivo `.aipo` é um módulo, resolvido como `<name>.aipo` ao lado do arquivo que o importa;
- `import m`, `import m: names`, `import m as alias` e `export` são todos respeitados, e
  as dependências são carregadas antes dos dependentes, de modo que a ordem de inicialização é topológica;
- **init once** vale sob imports repetidos (os statements de nível superior de um módulo executam uma vez);
- a privacidade é imposta pela resolução de nomes, não por um passe separado: uma declaração não exportada é
  renomeada para `<module>::<name>` dentro do seu próprio módulo, de modo que um importador que a nomeie recebe um
  `AIPO_SEM_EXPORT_UNKNOWN` comum, e a mesma rejeição se aplica ao alcançá-la pelo
  namespace do módulo (`m.secret()`), o que o caminho pelo namespace permitiria de outra forma;
- ciclos (`AIPO_SEM_IMPORT_CYCLE`) e módulos ausentes (`AIPO_SEM_UNKNOWN_MODULE`) são reportados
  como diagnósticos, e não como faults de runtime.

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
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

## Gate: `test`

```
$ cargo test --workspace
total_passed=136 total_failed=0
```

Suítes pertencentes ao S10:

| Suíte | Testes | O que fixa |
|---|---|---|
| `aipo-cli/tests/conformance.rs` | 13 | snapshots de programas, fixtures de falha, drift do formatter, exit codes, JSONL, casos de módulos |
| `aipo-cli/tests/fuzz_smoke.rs` | 3 | bytes aleatórios, programas mutados e programas truncados nunca causam panic no pipeline |
| `aipo-formatter` (`lib` + `tests/golden.rs`) | 6 + 4 | regras de renderização canônica e idempotência sobre o corpus de formatação |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
    Finished `dev` profile [unoptimized + debuginfo] target(s)
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## Gate: `documentation_impact`

- `docs/conformance/README.md` — layout do corpus, matriz de snapshots, workflow de regeneração.
- `docs/reference/cli.md` — já normativo; todo comando, flag e exit code documentado agora
  é sustentado por um teste, e não por intenção.
- `CHANGELOG.md` — entrada de CLI e formatter da Wave 1.

## Limitações conhecidas (registradas, não escondidas)

- `aipo fmt` renderiza a partir do stream de tokens e normaliza whitespace, indentação e
  espaçamento de operadores. Ele não reordena nem reescreve a sintaxe, e ainda não implementa `aipo fmt`
  sobre um diretório ou glob `*.aipo` além dos caminhos informados na linha de comando.
- Os argumentos nomeados são resolvidos apenas no ponto de chamada quando o callee é uma função declarada
  (veja o registro de evidência do S11); uma chamada por meio de um valor de função armazenado mantém a ordem do código-fonte.
