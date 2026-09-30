# Evidência — P00-G15 / Contratos Estáticos e Conformidade de Interfaces

**Goal:** `P00-G15` — relatório de contratos pré-execução, conformidade estrutural de interfaces e
as questões restantes do ADP-001
**Fase:** P00 (Foundation) · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Por que este goal existiu

O `P00-G14` fechou as duas lacunas de runtime encontradas pelo corpus do S11 e deixou três limites residuais por escrito:
`aipo-sema` não usava as anotações escritas para um relatório pré-execução, um contrato de interface era
aceito às cegas em vez de ser verificado, e as questões do ADP-001 sobre limites invertidos de `clamp`,
saturação de slice e fronteiras de construção NFC ainda estavam em aberto. Este goal fechou os três.

| Residual | Fonte da decisão no canon | Resultado |
|---|---|---|
| `aipo-sema` ignorava as anotações escritas | Language Reference §4: "uma incompatibilidade comprovável é diagnóstico antes da execução; uma violação descoberta somente em runtime é contract fault" | **fechado** |
| Um contrato de interface era aceito em vez de verificado | Sintaxe Canônica, *Interfaces e `satisfy`*: "interfaces continuam estruturais e `satisfy` continua uma promessa/verificação explícita" | **fechado** (com um defeito real corrigido) |
| ADP-001 Q3 — limites invertidos de `clamp` | Language Reference §4 (valor fora da faixa → `Failure`) vs. índice fora da faixa → fault | **fechado** — `Failure` recuperável |
| ADP-001 Q4 — slice fora da faixa | `Aipo Language — Especificação Viva`, modelo de `List`: "slices fora da faixa são tolerantes/clamped, ao contrário de índices exatos" | **fechado** — tolerante para `List`, `String` e `Bytes` |
| ADP-001 Q5 — onde o NFC é aplicado | Language Reference / Sintaxe Canônica: `String` é NFC "antes de ser exposta ao programa" | **fechado** — em toda fronteira de construção |

## Implementação

| Estágio | Mudança |
|---|---|
| `aipo-sema` | `DeclaredContract` coleta os contratos de parâmetros; `check_argument_contracts` confronta argumentos posicionais e nomeados com a declaração; `check_return_contract` executa onde o `return` está escrito; `literal_violates_contract` dispara apenas para uma incompatibilidade comprovável (um literal contra um contrato de tipo core, ou `none` contra um não nullable) |
| `aipo-diagnostics` | Novo código `AIPO_SEM_CONTRACT_VIOLATION_STATIC` (falha de linguagem, exit code 1) |
| `aipo-ir` | `interface_operations` agora emite a aridade **visível ao chamador** (o receiver não é um argumento no ponto de chamada), de modo que o contrato que o runtime impõe é o que a interface declara |
| `aipo-vm` | `value_diagnostic_name` reporta o nome de tipo declarado de uma struct nos diagnósticos de contrato, em vez do kind genérico de runtime |
| `aipo-lexer` | A decodificação de literais de string normaliza para NFC (todos os prefixos, strings multilinha e identificadores) |
| `aipo-vm` | A conversão `String(value)` e `String + String` (para o qual a interpolação faz lowering) normalizam para NFC |
| `aipo-stdlib` | `lower`, `upper`, `capitalize`, `replace`, `join` e `format` normalizam sua saída; `reverse` já o fazia |
| `aipo-cli` (testes) | O harness de conformidade serializa *toda* invocação da CLI em processo, e não apenas a que captura |

### O defeito de contrato de interface que este goal encontrou

A verificação estrutural foi entregue no `P00-G14`, mas estava errada de um jeito que importa: o builder
contava a aridade da interface **incluindo o receiver** (`fn draw(self)` → 1), enquanto o runtime
comparava a aridade que um ponto de chamada enxerga (0). Todo valor falhava no seu contrato de interface — inclusive os
que estavam em conformidade — e a mensagem do fault nomeava o kind genérico de runtime:

```
before — `render(Circle{r = 3})` on a conforming value
error: [AIPO_RT_TYPE_MISMATCH] contract violation at parameter `item`:
       expected Drawable.draw/1, got struct.draw/0
```

Agora ambos os lados usam a aridade visível ao chamador e o nome declarado do tipo que falha:

```
after — conforming value runs, non-conforming value faults clearly
$ aipo run docs/conformance/programs/16_interface_contracts.aipo
player ana
ana hp 6
player ana
nothing to draw

$ aipo run docs/conformance/diagnostics/17_runtime_interface_contract.aipo
circle 3
error: [AIPO_RT_TYPE_MISMATCH] runtime fault [AIPO_RT_TYPE_MISMATCH]: contract violation at
       parameter `item`: expected Drawable.draw/0, got Blank without 'draw'

$ aipo run docs/conformance/diagnostics/18_runtime_interface_arity.aipo
error: [AIPO_RT_TYPE_MISMATCH] runtime fault [AIPO_RT_TYPE_MISMATCH]: contract violation at
       parameter `item`: expected Drawable.draw/0, got Odd.draw/1
```

### A race no harness de conformidade que este goal encontrou

A regeneração dos snapshots (`AIPO_UPDATE_SNAPSHOTS=1`) produziu um `programs/07_strings_and_math.stdout` poluído
com um `3` inicial que a fixture nunca imprime. O sink de `io` é global ao processo e o harness antigo só
serializava a execução que *captura*, de modo que outro teste executando um programa concorrentemente escreveu sua saída
no buffer de captura. O lock agora cobre toda invocação da CLI, e o snapshot regenerado
coincide com um `aipo run` direto. Três execuções consecutivas da suíte estão verdes.

## Gate: `fmt` / `clippy`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0
(warnings: 0)
```

## Gate: `test`

```
$ cargo test --workspace
PASSED=146  FAILED=0

$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed
```

Nova certificação:

| Artefato | Comprova |
|---|---|
| `docs/conformance/diagnostics/15_sem_contract_violation.aipo` | um argumento literal que não pode satisfazer um contrato de parâmetro escrito é `AIPO_SEM_CONTRACT_VIOLATION_STATIC` antes da execução |
| `docs/conformance/diagnostics/16_sem_return_contract.aipo` | o mesmo para um literal retornado contra `-> T` |
| `docs/conformance/diagnostics/12` / `13` | uma incompatibilidade que o analisador não consegue provar continua sendo um fault de contrato em runtime |
| `docs/conformance/programs/16_interface_contracts.aipo` | uma interface como contrato escrito: `satisfy`, conformidade estrutural em runtime, `T?` aceitando `none`, uma operação com argumento |
| `docs/conformance/diagnostics/17_runtime_interface_contract.aipo` | um valor sem a operação é um fault de contrato que nomeia a struct |
| `docs/conformance/diagnostics/18_runtime_interface_arity.aipo` | uma operação de mesmo nome com aridade visível ao chamador errada é um fault de contrato |
| `docs/conformance/programs/17_unicode_nfc.aipo` | NFC em toda fronteira de construção: literal com escape, concatenação, interpolação, `join`, `replace`, `format`, mapeamento de caixa, `reverse`, e uma marca sem forma pré-composta que sobrevive |
| `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` | slices tolerantes para `List`, `String` e `Bytes`; `clamp` com limites invertidos recuperável via `or_else`; promoção numérica mista |
| `docs/conformance/diagnostics/19_runtime_clamp_inverted_bounds.aipo` | uma `Failure` de limites invertidos não tratada (`AIPO_RT_FAILURE_UNCAUGHT`) |
| `crates/aipo-lexer/src/lib.rs::test_string_literals_are_nfc_normalized` | o lexer armazena um par de escape decodificado já composto e mantém escapes brutos como brutos |
| `crates/aipo-vm/tests/data_and_errors.rs::test_string_concatenation_preserves_nfc` | `String + String` normaliza antes que o programa possa observar |
| `crates/aipo-stdlib/tests/stdlib_tests.rs::test_string_operations_preserve_nfc` | `join`, `replace`, `format` normalizam, `slice` preserva, e uma marca não componível sobrevive |
| `crates/aipo-vm/tests/data_and_errors.rs::test_interface_contract_accepts_a_conforming_operation` | um valor que expõe a operação satisfaz a interface |
| `crates/aipo-vm/tests/data_and_errors.rs::test_interface_contract_names_the_failing_struct` | o fault nomeia `Drawable.draw/0` e `Blank` |

Tamanho do corpus após este goal: 18 fixtures de programas, 19 fixtures de diagnósticos, 8 fixtures de formatação e
3 casos de módulos.

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
DOC_EXIT=0
(warnings: 0)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-001-byte-and-core-types-as-values.md` — Q3, Q4 e Q5 fechadas com a fonte no canon,
  a decisão e a fixture certificadora; o documento agora está totalmente resolvido.
- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G3 registra a conformidade estrutural
  de interfaces (e o defeito de aridade que foi corrigido); a lista de limites residuais não afirma mais que
  contratos de interface não são verificados.
- `docs/conformance/README.md` — matriz e inventário estendidos; a seção "lacunas verificadas" agora lista
  os três residuais como fechados.
- `docs/stdlib/mvp-subset.md` — contratos de interface, relatório estático de contratos, limites de `clamp`, a
  regra de slice e uma tabela dedicada de *Normalização de String e Unicode*.
- `CHANGELOG.md`, `PROJECT_STATE.md`, `docs/PRUMO.md`.

## Limitações conhecidas (registradas, não escondidas)

- **O relatório estático cobre apenas incompatibilidades comprováveis.** Um literal contra um contrato de tipo core e
  `none` contra um não nullable são comprováveis apenas a partir da expressão. Uma variável, o resultado de uma chamada ou o nome de uma
  `struct`/interface precisa do runtime, que continua sendo o canal de fault na fronteira da
  chamada. `aipo-sema` não é um type checker; ele reporta as contradições que consegue provar.
- **Um contrato que nomeia uma interface é verificado por nome e aridade, não por assinatura.** O canon torna
  as interfaces estruturais, e esta verificação confere que o valor expõe as operações declaradas com a
  aridade visível ao chamador declarada. Um nome coincidente com um tipo de parâmetro incompatível ainda só é
  descoberto quando a operação executa.
- **O NFC não é aplicado ao ler o constant pool do bytecode.** O lexer é o único produtor de constantes de string
  visíveis ao programa, então normalizar ali uma vez basta; isso volta a ser uma decisão real
  se uma fronteira de entrada de texto do host ou `Bytes.decode()` entrar no MVP.
- **`string.slice` e `List[start..end]` compartilham a regra tolerante; a indexação não.** O canon declara
  a regra tolerante para slices e a regra de fault para índices exatos; a extensão para slices de `String`
  e `Bytes` é a decisão documentada no ADP-001 Q4, e não uma citação do canon.
