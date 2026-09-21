# Changelog

Todas as alterações notáveis deste projeto são documentadas neste arquivo.
O formato baseia-se no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [Não lançado]

### Corrigido
- **Auditoria de docs (P01-G02)**: varredura completa de `docs/` — resíduo "Odin"
  confinado ao material histórico do canon (1 menção normativa corrigida sem
  mudança semântica); `scope.md`, `architecture/overview.md`, coding-standards,
  governance, deployment, observability, lifecycle, security-contract (paths +
  tabela `unsafe`), PRUMO.md, mvp-subset, status das waves e ADR-001 alinhados
  ao implementado (Waves 1–2, `aipo build`, sem GC, sem stdlib externa).
  114 links internos validados, zero quebrados. Registro em
  `docs/journal/2026-09-20-docs-audit.md`.

### Adicionado
- **Wave 3 combinadores assíncronos e scheduler (P02-G02)**: Scheduler cooperativo determinístico mono-thread com tempo virtual (`tick`), combinadores assíncronos (`task.spawn`, `task.sleep`, `task.all`, `task.race`, `task.timeout`, `task.cancel`, `task.group`), `group.spawn`, `group.wait`, suporte completo a `await` e detecção de ciclo de await (`AIPO_RT_AWAIT_CYCLE`), restrição de bloqueio em callbacks de host (`AIPO_RT_AWAIT_IN_CALLBACK`) e 100% de paridade diferencial VM↔JS (`aipo-runtime.js`).
- **Wave 3 tipos e valores (P02-G01)**: `Set` ordenado com semântica de conjunto (`has`, `add`, `remove`, `clear`, `to_list`, `lazy`); `Sequence` lazy iterável com pipeline (`map`, `filter`, `flat_map`, `take`, `skip`, `distinct`, `enumerate`) e terminais (`collect`, `find`, `any`, `all`, `count`, `reduce`, `group_by`); `Bytes` com buffer mutável (`Rc<RefCell<Vec<u8>>>`) e packing binário little-endian (`read_*`/`write_*` para `i8`..`f64`), mais `String.encode()` e `Bytes.decode()`; `Duration` com precisão em segundos, aritmética (`+`, `-`, comparação) e `.total_seconds()`; handles de `Task` e `Group` no modelo de valores com combinadores do módulo `task` (`spawn`, `sleep`, `all`, `race`, `timeout`, `cancel`, `group`); paridade estrita entre VM Rust e runtime JavaScript (`aipo-runtime.js`) comprovada pela suíte diferencial e selftest.
- **Deep quality gauntlet (P01-G02)**: `aipo-testkit` (Rng, AipoSmith generator,
  differential/metamorphic harnesses, portable subprocess watchdog),
  `aipo-bench` (frontend/VM/JS/scaling baselines → `docs/performance/baseline.md`),
  libFuzzer targets (`fuzz/`, nightly tier), LLVM coverage (`docs/testing/coverage.sh`),
  property suites (lexer, source, parser totality, values, contracts, unicode, fmt),
  UI diagnostic goldens + accessibility rubric/protocol, source-map conformance,
  determinism suite, resource-exhaustion suite, examples 06–24 with executable
  harness, programs 21–23, `deny.toml` supply-chain policy, ADPs 003/004/005.
  Suite: 165 → 253 testes, zero falhas; Miri verde em `aipo-vm`; MSRV 1.85 verificado.

### Corrigido
- **Abort do host em nesting profundo (SIGABRT)**: guardas de profundidade no
  parser + garantia de progresso + `AIPO_PARSE_NESTING_TOO_DEEP` (ADP-005);
  latente: loops de corpo travavam em qualquer falha sem consumo.
- **Panic do IR builder em programa válido**: construção com `init` dentro de
  função com parâmetros (achado do fuzzer); usa `hidden_name` (regressão: programa 23).
- **Divergência VM↔JS no recovery de `attempt`**: journal truncado até o frame
  sobrevivente (regressão: programa 21).
- **Divergência de zero com sinal**: `-0.0` no display e normalização de `Int`
  negativo inexistente no JS (regressão: programa 22).
- **Parsing quadrático de f-strings**: slices sem padding + shift de spans
  (`shift.rs`), 23,4 ms → 1,8 ms em 400 f-strings, snapshots idênticos.
- **Examples apodrecidos**: 03 (`return` em `invariant`), 04 (`fold`
  inexistente, pipelines liderando linha).
- **Fixture `programs/21_attempt_recovery_and_journal`**: `return fail(…)` propagando até o `attempt` do chamador e recovery descartando entradas do journal anteriores ao handler (só mutações pós-handler revertem) — cobre os dois backends.
- **Índice de chaves `String` no `Dict`** (`DictMap`): lookup/upsert O(1) no caso comum, preservando ordem de inserção e igualdade estrutural; espelhado no shim JS.
- **Fuzz gramatical** (`fuzz_smoke`): mutações por tokens (keywords, `end`-stripping, splice de programas) sobre `check`/`fmt --check`, mais execução real de mutantes sob `timeout` com assert de exit codes e ausência de panic.
- **Property tests do `aipo-js`**: determinismo da emissão e coerência do bundle sobre todo o corpus, mais `runtime/selftest.mjs` (propriedades da camada pura do shim sob `node`).
- **Documentação de distribuição**: `README.md` reescrito (quickstart, comandos, backends, testes), `CONTRIBUTING.md`, licenças `LICENSE-MIT`/`LICENSE-APACHE` (dual license do workspace).

### Corrigido
- **Emissor rejeita estouros de operandos**: contagens que não cabem em u16/u8 (nomes, constantes, itens, aridade, slots, capturas, alvos de salto) e funções desconhecidas agora são erro de compilação em vez de truncamento silencioso.
- **Paridade do journal no `attempt` (JS)**: o recovery passa a descartar até o `journal_start` do frame sobrevivente, como a VM — antes entradas pré-handler sobreviviam e o rollback divergia (fixture 21 prova).
- **Mensagem de `Overflow`**: inclui frames ativos e a causa usual (recursão profunda), nos dois backends.

### Removido
- **Variante morta `VmFault::InvalidDictKey` e código `AIPO_RT_INVALID_DICT_KEY`**: nada a produzia (qualquer `Value` é chave válida com igualdade `==`); catálogo atualizado.

### Refatorado
- **`aipo-vm/src/vm.rs` fatiado em `src/vm/`** (`mod`, `dispatch`, `call`, `journal`, `contract`, `method`, `failure`, `helpers`) sem mudança semântica — suíte completa verde antes e depois.
- **Contrato `aipo-vm`**: threading single-thread documentado como decisão (`Rc<RefCell>` ⇒ `!Send`), menções obsoletas a `gc-arena` removidas.

### Corrigido
- **Escopo de módulo (P00-G16)**: bindings `let`/`var` de topo de módulo agora são visíveis dentro de funções, métodos e `impl` — o padrão canônico `var counter` + `fn bump()` compila e executa. Causa: o `aipo-sema` analisava corpos de itens antes de declarar os bindings de topo.

### Adicionado
- **Funções locais (P00-G16)**: `fn nome(params) ... end` declarado dentro de uma função agora cria um binding local (canon: "Funções locais e closures — decidido"). O binding existe quando a execução alcança a declaração, o nome é visível no próprio corpo para autorrecursão e capturas lexicais seguem `let`/`var` (captura compartilhada de `var`). Novo opcode `FillSelfCapture` completa o handle de recursão: a captura de si mesma nasce com o sentinel `Unset` e é reescrita com o closure recém-criado. Fixtures `programs/19_local_functions` e `programs/20_module_scope`.
- **Wave 1 exit review (P00-G16)**: auditoria dos critérios de saída em `docs/waves/wave-1-mvp.md` contra o corpus, gauntlet 100% (conformance 13/13, formatter 11/11, fuzz 3/3, workspace 146/146), inventário verificado (20 programas, 19 diagnósticos, 8 formatações, 3 módulos) e non-delivery explícito (`Bytes` packing, `aipo-js`, `Set`/`Sequence`, LSP/REPL). Hand-off em `docs/evidence/P00-G16-wave-1-exit-review.md`.

## [0.1.0] - 2026-09-15

### Adicionado
- **Slice S1 (P00-G01)**:
  - Crate `aipo-source`: normalização UTF-8 NFC/BOM/CRLF, codificação de spans, cálculo de linha e coluna.
  - Crate `aipo-diagnostics`: catálogo estável de diagnósticos, emissores humano e JSON Lines (`--message-format=jsonl`).
- **Slice S2 (P00-G02)**:
  - Crate `aipo-lexer`: análise léxica completa, números sem avaliação precoce, strings (`f`, `r`, `fr`, `"""`), pipeline `|>`.
- **Slice S3 (P00-G03)**:
  - Crate `aipo-ast`: AST fortemente tipada para itens, declarações e expressões com spans preservados.
  - Crate `aipo-syntax`: parser recursivo descendente, parser Pratt de expressões, suporte a trailing blocks e recuperação de erros.
- **Slice S4 (P00-G04)**:
  - Crate `aipo-hir`: High-Level IR com desaçucaramento semântico de blocos em cauda (`do ... end`), pipelines (`|>`), e desestruturação em bindings.
- **Slice S5 (P00-G05)**:
  - Crate `aipo-sema`: resolução de escopos léxicos, verificação de caminhos de mutabilidade (`let` vs `var`, `!`), checagem estática de aridade e conformidade de interfaces (`satisfy`).
- **Slice S6 (P00-G06)**:
  - Crate `aipo-ir`: Core IR neutro de target com instruções sequenciais e preparação de pool de constantes.
  - Crate `aipo-bytecode`: conjunto compacto de opcodes, emissor, formato binário `aibc` v1, verificador estrutural e desassemblador.
- **Slice S7 (P00-G07)**:
  - Crate `aipo-vm`: máquina de pilha (stack machine) com despacho de instruções de bytecode, molduras de chamada (`CallFrame`), modelo de valores (`Value`), checagem estrita da faixa de inteiros ±(2^53 - 1) (`AIPO_RT_OVERFLOW`), floats finitos (`AIPO_RT_NON_FINITE_FLOAT`), divisão por zero (`AIPO_RT_DIV_ZERO`), variáveis globais e locais, e saltos condicionais/incondicionais.
- **Slice S8 (P00-G08)**:
  - Crate `aipo-vm`: coleções ordenadas `List` e `Dict` com suporte a indexação negativa, instanciação de `Struct` com imutabilidade de campos `fixed` e validação de `invariant()`, propagação automática de `Failure` (Modelo B), recuperação com `or_else` e blocos `attempt ... failed err ... end`, e diferenciação normativa de runtime faults incondicionais.
- **Slice S9 (P00-G09)**:
  - Crate `aipo-runtime`: registro de módulos (`ModuleGraph`, `ModuleRecord`, `ModuleState`), ordem de inicialização topológica determinística com desempate lexicográfico pelo caminho canônico, detecção de import circular (`AIPO_SEM_IMPORT_CYCLE`) e de dependência ausente (`AIPO_SEM_UNKNOWN_MODULE`), e catálogo de funções nativas (`NativeRegistry`).
  - Crate `aipo-stdlib`: Prelude V1 (`none`, `true`, `false`; `len`, `copy`, `same`, `some`, `fail`; conversões explícitas `Int`, `Float`, `Byte`, `String`), módulo `math` (`abs`, `min`, `max`, `floor`, `ceil`, `round` meio-para-longe-de-zero, `truncate`, `sqrt`, `pow`, `clamp`, constantes `pi`/`e`), módulo `string` (`len`, `byte_len`, `contains`, `starts_with`, `ends_with`, `find`, `lower`, `upper`, `capitalize`, `reverse` por grafemas com renormação NFC, `trim`, `split`, `join`, `replace`, `slice`, `format` com placeholders nomeados) e módulo `io` (`print`, `println` com sink capturável).
  - Regras de canon aplicadas e testadas: `split`/`replace` rejeitam padrão vazio, `join` recusa coerção textual implícita, campos vazios preservados, parsing textual estrito nas conversões, e separação estrita entre `Failure` recuperável (Modelo B) e runtime fault.
- **Completude de Backend (P00-G10)**:
  - Crate `aipo-vm`: valores de função de primeira classe e closures com upvalues, novos kinds `Byte`, `Bytes` e `Type`, testes de tipo, indexação por `Range` em `List`/`String`/`Bytes`, molduras de chamada cientes do receptor e enforcement de `AIPO_RT_MUTATION_DURING_ITERATION`.
  - Crate `aipo-bytecode`: tabela de `struct` no módulo binário, novos opcodes para parâmetros com default e prólogo callee-side, emissor, verificador e desassemblador cobrindo todo o conjunto.
  - Crate `aipo-ir`: declarações de `struct` carregadas no `Program`, construção `Type{...}` com campos em ordem canônica e resolução de argumentos nomeados no call site.
  - Crate `aipo-stdlib`: métodos de `List`/`Dict` ligados à VM (incluindo os de ordem superior `transform`, `filter` e `sort_by`, que chamam de volta em Aipo) e conversões delegando à VM para que `String(v)` e `io.print(v)` não divirjam.
  - Crate `aipo-syntax`: interpolação `f"..."` dessucarada para concatenação com `String(...)` e slices com limites omitidos (`a[..b]`, `a[a..]`, `a[..]`).
  - Crate `aipo-cli`: execução de módulos `import`/`export` com ordem topológica, init-once, privacidade por resolução de nomes, `AIPO_SEM_IMPORT_CYCLE` e `AIPO_SEM_UNKNOWN_MODULE`.
- **Slice S10 (P00-G11)**:
  - Crate `aipo-formatter`: formatação determinística e idempotente a partir do fluxo de tokens (indentação de quatro espaços, alinhamento de `end`, espaçamento de operadores, preservação de comentários).
  - Crate `aipo-cli`: superfície `aipo run <path>`, `aipo check <path>`, `aipo fmt <paths...> [--check]`, `aipo --version`, `aipo --help`, com códigos de saída `0`/`1`/`2` e diagnósticos em `--message-format=human|jsonl`.
- **Slice S11 (P00-G12)**:
  - Corpus de conformidade `docs/conformance/`: 11 programas com stdout commitado, 10 fixtures de diagnóstico com código esperado, 8 pares golden do formatador e 3 casos de módulos.
  - Suíte `aipo-cli` de conformidade (13 testes), fuzz smoke sem panic sobre o pipeline (3 testes) e rubrica de gauntlet documentada em `docs/conformance/README.md`.
  - Lacunas verificadas registradas em `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` (`init`, `invariant()`, contratos em runtime e construção de `Bytes`) e encaminhadas para `P00-G13`.
- **Fechamento de contratos do MVP (P00-G13)**:
  - `Type{...}` passa a executar o hook `init` quando a `struct` declara um: o parser injeta o receiver implícito `self!`, o Core IR emite `BuildStruct` → chamada `Type.init` (argumentos alinhados à declaração, defaults pelo prólogo callee-side) → `Pop` → verificação de `invariant()` → `SealStruct`.
  - Campos `fixed` podem receber valor durante a construção (inclusive por `init`) e só são imutáveis depois da publicação (`StructInstance::under_construction` + opcode `SealStruct`).
  - `invariant()` é lowered para um predicado `<Type>.invariant(self) -> Bool` com as linhas combinadas por `and` e verificado ao fim da construção; `AssertInvariant` converte o resultado falso em `VmFault::InvariantViolation` (fault de contrato, conforme o error model do recorte MVP).
  - `Bytes(count)` passa a ser a forma canônica de construção de `Bytes` (bloco zerado, indexação por byte produzindo `Byte`, `len`), com limite provisório de alocação e `Failure` recuperável fora dele.
  - Corpus atualizado: `programs/12_bytes.aipo`, `programs/13_init_and_invariant.aipo` e `diagnostics/11_runtime_invariant_violation.aipo`.
- **Runtime contract enforcement (P00-G14)**:
  - `invariant()` passa a ser reavaliado nas fronteiras mutáveis estáveis (canon: `candidate -> aplicação provisória -> verificação -> commit`). Campos de instância publicada entram num *journal* por frame (`SetField` aplica provisoriamente e guarda o valor de entrada); o novo opcode `CheckMutations` verifica todas as instâncias participantes na fronteira — retorno de função/método ou statement de mutação do script de entrada — e, em falha, restaura os campos diretos de todas elas e produz uma `Failure` recuperável (capturável por `attempt`), preservando o valor de entrada.
  - O predicado compilado `<Type>.invariant` é resolvido por tipo a partir da tabela de funções do próprio módulo (`Vm::run`), então a verificação de mutação e a de construção usam o mesmo código, sem um segundo avaliador de expressões.
  - Contratos de assinatura (`name: Type`, `name!: Type`, `-> T`, `T?`) passam a ser verificados em runtime pelo novo opcode `AssertContract`: parâmetros no prólogo do callee (depois do prólogo de defaults, cobrindo também o valor de um default) e retornos antes de cada `return expr`. `none` só satisfaz `T?`; `Function` exige valor chamável; o nome de uma `struct` é comparado com o tipo da instância; uma violação descoberta em runtime é `VmFault::ContractViolation` (fault, não capturável por `attempt`); um contrato que nomeia uma interface é aceito, porque o recorte MVP não tem checagem estrutural em runtime.
  - Corpus atualizado: `programs/14_signature_contracts.aipo`, `programs/15_invariant_on_mutation.aipo`, `diagnostics/12_runtime_contract_violation.aipo`, `diagnostics/13_runtime_return_contract.aipo` e `diagnostics/14_runtime_invariant_mutation_uncaught.aipo`.
  - `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` fica **resolved**: os cinco gaps encontrados pelo corpus de S11 (G1, G2, G2b, G3, G4) estão fechados e certificados.
- **Static contract reporting e conformidade estrutural de interfaces (P00-G15)**:
  - `aipo-sema` passa a usar as anotações escritas para um relatório **antes da execução**: um argumento literal que não pode satisfazer um contrato de parâmetro de tipo core, ou `none` contra um contrato não anulável, vira o novo diagnóstico `AIPO_SEM_CONTRACT_VIOLATION_STATIC` (também para `return` literal contra `-> T`). Um descompasso que o analyzer não consegue provar continua sendo contract fault em runtime, no limite da chamada.
  - Conformidade estrutural de interface em runtime deixa de ser aceitação cega: o contrato carrega as operações que a interface declara e checa se o valor as expõe. **Correção de defeito real**: a aridade emitida passou a ser a visível no call site (o receiver não é argumento ali), alinhada à convenção já usada por métodos nativos e `BoundMethod`; antes a interface contava o receiver e reprovava até os valores conformes. A mensagem de falha passa a nomear o tipo declarado da `struct` em vez do kind genérico do runtime.
  - NFC passa a ser garantida nas fronteiras de construção de `String` (ADP-001 Q5): decodificação de literal no lexer, conversão `String(value)`, concatenação `+` (para onde a interpolação abaixa) e as operações que podem juntar base + marca (`lower`, `upper`, `capitalize`, `replace`, `join`, `format`). Operações puramente substring preservam o invariante de graça; `==` continua igualdade estrutural.
  - ADP-001 Q3 fechada: `math.clamp` com limites invertidos permanece `Failure` recuperável, pelo critério que o próprio canon usa para separar valor fora de faixa (`Failure`) de índice fora de faixa (fault). ADP-001 Q4 fechada: slice fora da faixa é tolerante para `List`, `String` e `Bytes` (canon: "slices fora da faixa são tolerantes/clamped, ao contrário de índices exatos").
  - Correção de flakiness na suíte de conformidade: o sink de `io` é global, então a suíte passa a serializar **toda** invocação do CLI, não só a que captura — antes outra execução concorrente contaminava um snapshot regenerado.
  - Corpus atualizado: `programs/16_interface_contracts.aipo`, `programs/17_unicode_nfc.aipo`, `programs/18_tolerant_slices_and_clamp.aipo`, `diagnostics/15_sem_contract_violation.aipo`, `diagnostics/16_sem_return_contract.aipo`, `diagnostics/17_runtime_interface_contract.aipo`, `diagnostics/18_runtime_interface_arity.aipo` e `diagnostics/19_runtime_clamp_inverted_bounds.aipo` (18 programas, 19 diagnósticos, 8 formatações e 3 casos de módulo).
  - `docs/adp/ADP-001-byte-and-core-types-as-values.md` fica **resolved**: todas as perguntas (Q1–Q5) fechadas, cada uma com a fonte no canon, a decisão e o fixture que certifica.
- Bateria de testes unitários exaustivos cobrindo todo o pipeline (146 testes automatizados 100% aprovados, zero warnings em lints/clippy e `unsafe_code = forbid`).
