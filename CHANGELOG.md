# Changelog

Todas as alterações notáveis deste projeto são documentadas neste arquivo.
O formato baseia-se no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [0.11.0] - Em desenvolvimento (Trilha WebAssembly & Self-Hosting)

- **Transição Arquitetural para WebAssembly e Roteiro de Self-Hosting (ADP-013)**:
  - Congelamento formal da versão inicial v0.1.0 (Stack VM) na branch `legacy/v0.1.0-stack-vm` e tag `v0.1.0-stack-vm-final`.
  - Adoção do WebAssembly (Wasm 2.0 / WASI) como substrato padrão de execução de alta performance, superando o piso de ~100ns do despacho em pilha para ~1-3ns com JIT nativo via Cranelift/Wasmtime.
  - Setup inicial da crate `aipo-wasm` integrada ao workspace para emissão limpa de arquivos binários `.wasm` via `wasm-encoder`.
  - Preservação integral de 100% das camadas de frontend (`aipo-source`, `aipo-diagnostics`, `aipo-lexer`, `aipo-syntax`, `aipo-hir`, `aipo-sema`).
  - Estabelecimento do roteiro de self-hosting em 3 fases, habilitando o futuro compilador auto-hospedado da Aipo sem aprisionamento em FFI.

## [0.1.0] - 2026-09-26 (Linha de Base Stack VM)

- **Pivot Ergonômico de Sintaxe e Filosofia de Design (ADP-012)**:
  - **Blocos Delimitados por Chaves (`{ ... }`)**: Adoção de blocos explícitos com `{ ... }` em todas as estruturas de controle (`if`, `while`, `loop`, `repeat`, `each`, `match`, `attempt`, `await do`) e definições de itens (`fn`, `struct`, `impl`, `init`, `invariant`, `interface`), eliminando a necessidade de parênteses em condições e abolindo o ruído visual da palavra-chave `end`.
  - **Imutabilidade de Campos de Struct por Padrão**: Campos declarados de forma simples em `struct` (`id`, `created_at`) são imutáveis por padrão. Mutabilidade exige prefixo explícito `var` (`var status = "idle"`). A palavra-chave `fixed` foi extinta.
  - **Instanciação Simétrica de Struct com Dois-Pontos (`:`)**: Padronização da inicialização de structs usando pares chave-valor idênticos aos dicionários (`User{ name: "Dev", age: 30 }`), unificando o modelo mental da linguagem.
  - **Mutabilidade Universal de Receptores (`var self`)**: Substituição de `self!` por `var self` em métodos e hooks de invariante, preservando conformidade com a declaração de variáveis mutáveis da linguagem.
  - **Subtipagem Estrutural Automática**: Structs satisfazem interfaces implicitamente quando suas assinaturas de método conferem, tornando o top-level `satisfy Type: Interface` opcional (útil como asserção explícita de compilação).
  - **Divisão Inteira Truncada (`//` e `//=`)**: Adoção de `//` e `//=` para divisão inteira truncada em direção a zero, substituindo `div` e `div=`.
  - **Formatter Canônico (`aipo-formatter`)**: Atualizado para suportar e formatar canonicamente blocos com `{ ... }`, espaçamento interno ergonômico de chaves em uma linha e alinhamento idempotente.
  - **Documentação e Exemplos Multi-idiomas**: 100% da documentação oficial (em Português e Inglês) e todos os 24 programas em `examples/` atualizados e validados com 100% de paridade entre a VM e o backend JavaScript.
- **Recorte da `aipo v0.1.0` (ADP-008)**: a primeira release é da linguagem, com async, CLI mínima, test runner, C ABI síncrona e thin proofs de interoperabilidade Rust/C/JS; engines, editors, registry e web profile completo ficam pós-v1.
- **Test Runner Canônico `aipo test` (ADP-008, ADP-012)**:
  - Implementado o comando `aipo test` no CLI para descoberta automática recursiva de testes em arquivos `*_test.aipo` e `test_*.aipo`.
  - Suporte a filtros de teste (`--filter <substr>`), saída formatada para humanos e streaming NDJSON legível por máquinas (`--message-format <human|jsonl>`) e `--package-cache`.
  - Isolamento estrito por teste: cada caso de teste roda em uma instância limpa da VM, com congelamento do relógio (`DeterministicClock`) e reset da semente do PRNG para zero (`reset_default_seed(0)`), garantindo determinismo temporal e numérico absoluto.
  - Interceptação nativa do subcomando de testes através de `TestMode::Discover` e `TestMode::Execute` em `aipo-vm`, com correspondente paridade diferencial em `aipo-js` (`setTestMode`, `testCall`).
- **C ABI Estável e Embedding Interface (`aipo-c-abi`, ADP-009, ADP-010)**:
  - Crate dedicada `crates/aipo-c-abi` e cabeçalho canônico C `crates/aipo-c-abi/include/aipo.h` para consumo direto em C, C++, Python ctypes, Go cgo e outras linguagens hospedeiras.
  - ABI estável, estritamente síncrona e single-threaded (`aipo_runtime_t`), sem promessa de concorrência interna, garantindo integridade sem overhead de locks.
  - Blindagem total contra panics do Rust: todas as funções exportadas são encapsuladas em `catch_unwind(AssertUnwindSafe(...))`, mapeando falhas internas para códigos C (`AIPO_ERR_FAULT`).
  - Representação desacoplada de valores em C (`aipo_value_t`), mantendo os invariantes de float finito e inteiros no intervalo seguro ±(2^53 - 1).
  - Handles geracionais do host (`aipo_handle_t`) validados por índice de slot e contador de geração; acesso após liberação retorna `AIPO_ERR_STALE_HANDLE` sem corrupção de memória.
  - Registro de funções nativas de host (`aipo_host_fn_t`), com controle de acesso baseado em capacidades (`grant`/`revoke`), retornando `AIPO_ERR_CAPABILITY_DENIED` quando não autorizadas.
  - Mapeamento transparente de falhas recuperáveis (Model B / `AIPO_ERR_UNCAUGHT_FAILURE`), erros diagnósticos (`AIPO_ERR_DIAGNOSTIC`), e recuperação do buffer do último erro (`aipo_last_error`).
- **Roteiro de Performance Estrutural e Ergonomia de Linguagem (ADP-011)**: formalizada a estratégia de superação do piso de despacho (~100ns/opcode) através de compactação de `Value` (48B -> 16B/24B), invocação fundida `InvokeMethod` (sem alocação de `BoundMethod`), laço de despacho com variáveis hoisted e superinstruções `Call0..Call4`/`GetLocal0..GetLocal3`; além da expansão ergonômica com falhas estruturadas tipadas (Model B+), pattern matching e type narrowing estático em `aipo-sema`.
- **Performance — Compactação de Layout de `Value` (P1)**:
  - Redução do tamanho da enum central `Value` de 40 bytes para 24 bytes (redução de 40% em memória por slot), alinhando o descritor de chamada e economizando largura de banda de cache L1/L2.
  - Validação empírica A/B comprovou ganho de **-13,94% na mediana de tempo de CPU** em workloads intensivos de pilha e chamadas de função.
- **Performance — Superinstruções `GetLocal0..7` e `Call0..4` (P4)**:
  - Introduzidos opcodes de byte único `GetLocal0..GetLocal7` (60..67) e `Call0..Call4` (68..72), eliminando decodificação de operandos Big-Endian no caminho quente da VM.
  - A VM rastreia a largura dinâmica da instrução de chamada (`call_inst_len`), garantindo que suspensões cooperativas e temporizadas (`task.sleep`, `task.timeout`, `task.all`, `race`) retrocedam o IP com exatidão matemática.
  - Validação empírica A/B via `scripts/perf/cpu_ab.py` com 8 rounds e 8 reps atestou **-8,86% no tempo mínimo de CPU** e **-6,49% na mediana** na suíte completa (`fields, arithmetic, recursion, collections`).
- **Performance — Despacho Direto de Métodos de Structs (P2)**:
  - Adicionada a variante leve `Value::StructMethod { receiver, entry_ip, total_arity, is_async }` e registro monomórfico na VM, eliminando a instanciação transitória de `Rc<BoundMethodData>` no heap.
  - Elimina 200.000 alocações no heap por 100k chamadas no benchmark `fields`, com ganho comprovado de **-7,54% na mediana de CPU**.
- **Ergonomia — Falhas Estruturadas Tipadas (E1 / Model B+)**:
  - `FailureValue` agora carrega `payload: Value`, suportando construtores `FailureValue::new` e `with_payload`, e construtores correspondentes em `Value`.
  - Acesso a `err.payload` suportado nativamente na VM e na função nativa `fail` da stdlib; paridade diferencial de 100% mantida no backend JavaScript (`aipo-js`).

### Adicionado
- **Performance — Cross-language benchmark suite**:
  - `aipo-bench --compare` compara workloads determinísticos com Aipo CLI/VM, Aipo VM in-process, Aipo→JavaScript/Node, Lua, LuaJIT, Wren, Luau, CPython, PyPy, Ruby, JavaScript/Node e Rust nativo.
  - O runner exige checksums iguais, executa as mesmas operações lógicas no Rust nativo, mede warmup separadamente e grava samples brutos, median/MAD, p95, versões, profile e estado do Git em JSON.
  - `--compare-workloads` seleciona workloads individualmente para A/B isolado, sem misturar o custo de outras operações.
  - Rust nativo é controle de lower bound; startup, compilação e steady-state são reportados separadamente, com limites explícitos de entrada e rondas.
  - A primeira onda adiciona Wren 0.4.0, Luau 0.739 e PyPy 8.0.0 como referências opcionais, com fixtures portáveis, candidatos de executável, argumentos específicos do Luau e fontes pinadas em `aipo-reference-runtime/SOURCES.json`.
  - O workload `fields` mede seis campos, mutação e dispatch de método; a primeira evidência directional mostrou Aipo VM em 1.430,92ms contra 120,35ms do Wren e 63,15ms do Luau, motivando uma otimização de lookup de fields como próximo experimento.
  - **Performance — Cache monomórfico de field slot**: `aipo-vm` memoriza o slot por site e tipo, acelerando `GetField`/`SetField` sem alterar o layout público de `StructInstance`; A/B de 15 samples reduziu o median de 1.001,77ms para 718,20ms (-28,31%) no workload `fields`, com 2.399.992 hits e 8 misses.
  - **Performance — Cache do frame base**: `aipo-vm` mantém o `stack_base` do frame ativo para evitar `frames.last()` em cada acesso a local; A/B pareado de 31 samples reduziu `arithmetic` de 276,23ms para 265,98ms (-3,71%), `fields` de 736,02ms para 687,05ms (-6,65%) e `recursion` de 97,72ms para 93,65ms (-4,17%), com checksums e métricas idênticos. A validade da restauração após retorno aninhado e troca de tasks tem teste de regressão dedicado.
  - **Performance — `SetField` sem cópia de entrada para structs unguarded**: a cópia do valor anterior agora ocorre somente quando o tipo é guarded e precisa entrar no journal; o fixture `fields` elimina 600.000 clones de `Value` de forma determinística. O A/B de 31 samples não mostrou ganho consistente de wall-clock, portanto não há claim de velocidade.
  - O relatório agora usa schema 5, expõe `field_cache_hits`/`field_cache_misses` e pode registrar `peak_rss_bytes` por amostragem Linux com `--compare-resources`, em uma execução separada; contadores de alocação permanecem nulos até adapters runtime-specific existirem.
  - **Performance — runner pareado dedicado**: `scripts/perf/paired.sh` fixa CPU, alterna baseline/candidate, exige `taskset` por padrão e grava relatórios com manifest de proveniência sem modificar o estado do Git.
  - **Performance — entry de cache com guardedness**: o cache por site foi estendido para guardar `type_is_guarded`, mas o A/B pareado não mostrou ganho (+6,32% em `arithmetic`, +1,40% em `fields`, +2,13% em `recursion`) e a extensão foi revertida.
  - **Performance — cache de flags `fixed` por slot**: o `Vec<bool>` experimental foi mais lento no A/B pareado (+5,34% em `arithmetic`, +11,32% em `fields`, +144,36% em `recursion`) e foi revertido.
  - **Performance — cache de tipos guarded**: o `HashSet<String>` experimental foi mais lento no A/B pareado (+6,58% em `arithmetic`, +1,63% em `fields`, +3,36% em `recursion`) e foi revertido.
  - **Performance — layout denso de `StructInstance`**: separar `field_names` de `fields` (em vez de `Vec<(String, Value)>`) piorou `fields` nos três pares do A/B pareado (+3,73%, +7,39% e +24,23%), em parte por trocar uma alocação por instância de struct por duas. Como o experimento também era API-breaking, foi revertido e a API pública permanece inalterada.
  - **Performance — despacho de método sem alocar**: um índice aninhado para `struct_methods` eliminou quatro alocações de `String` por bind de método, mas o A/B de quatro lotes não demonstrou ganho: a mesma mudança oscilou de -5,6% a +4,5% conforme a posição do novo campo na struct `Vm`. O protótipo foi revertido para não duplicar estado de registro sem benefício.
  - **Performance — despacho de opcode sem `Result` largo**: `Vm::step` decodificava cada opcode com `try_from(..).map_err(..)?`, construindo um `Result` de 72 bytes (o tamanho de `VmFault`) e um closure que capturava `self` em **todo** opcode executado. O `match` equivalente só constrói a falha no ramo frio, com offset e mensagem idênticos, e um teste novo fixa esse erro. A medição por tempo de CPU com 24 repetições alternadas deu resultado **nulo** (min 3,019s contra 3,014s, -0,16%), porque o compilador já afundava a construção da falha para fora do caminho feliz. A alteração foi mantida como robustez e clareza, não como otimização: o caminho de erro frio passa a ser explícito e o contrato de `unknown opcode` ganhou teste.
  - **Metodologia — piso de ruído do runner**: o host entrega aproximadamente ±5% de ruído em `fields` e `arithmetic` em condições normais, e até 2,5x sob contenção; diferenças menores que isso não são evidência, e `arithmetic` passa a ser usado como workload de controle por não tocar o caminho alterado. `scripts/perf/cpu_ab.py` compara tempo de CPU do processo filho em vez de wall-clock, que é o estimador robusto sob disputa.
- **Performance — Otimizações do caminho quente da VM e do shim JS**:
  - `aipo-vm` passou a usar cache de constantes, slots de globals e índices de campos/métodos, com fallback para structs nativas não registradas; natives e conversões também recebem argumentos por slice emprestado, removendo `Vec` por chamada.
  - `aipo-bytecode` deduplica constantes e remove propagações de falha redundantes após constantes e functions; strings ASCII têm fast path para concatenação e indexação.
  - `aipo-js` prepara locals/upvalues por slot no carregamento do módulo, usa cache preguiçoso de constantes por instrução e mantém o contrato de paridade; métricas da VM são coletadas fora do tempo de execução no relatório cross-language.
  - A rodada local final registrou 45 resultados, 0 skipped e 0 failures; o relatório completo permanece em `target/cross-language-final.json`, e a medição pós-mudança sob carga elevada está em `target/cross-language-final-current.json`.
- **CLI — Autenticação GitHub opt-in (P04-G11)**:
  - `aipo package fetch-github` e `aipo package lock --fetch-github` aceitam `--github-token-env <name>`; o valor do token vem exclusivamente da variável de ambiente nomeada.
  - Credenciais bearer são validadas, redacted em `Debug` e nunca são aceitas diretamente como argumentos, persistidas, logadas ou incluídas em diagnostics; sem a flag, o comportamento público sem `Authorization` permanece o padrão.
  - Tokens ausentes, vazios ou com whitespace/caracteres de controle falham com `AIPO_PKG_FETCH` antes de qualquer request; redirects continuam bloqueados.
- **CLI — Prune explícito do cache (P04-G10)**:
  - `aipo package cache prune <dir> --lock <lockfile>` é dry-run por padrão e lista somente entradas verificadas não referenciadas pelo lockfile.
  - `--apply` é obrigatório para remover; qualquer entrada inválida bloqueia toda a operação, sources referenciadas são preservadas e o lockfile nunca é alterado.
  - A política não usa rede, autenticação ou registry e não remove entradas corruptas/desconhecidas.
- **CLI — Verificação read-only do cache (P04-G09)**:
  - `aipo package cache verify <dir>` audita todas as entradas GitHub existentes, validando symlinks, arquivos regulares, metadata, source/key, manifest e digests SHA-256.
  - Entradas corruptas são reportadas como `AIPO_PKG_FETCH`; cache ausente ou vazio retorna sucesso com zero entradas.
  - A operação está disponível no build padrão, não acessa a rede e nunca cria, repara, remove ou faz prune no cache.
- **CLI — Raízes locais com dependências GitHub pinadas (P04-G08)**:
  - `aipo package lock <dir> --fetch-github --cache <dir>` resolve branches `path` locais e edges GitHub pinadas em um lockfile determinístico único, usando o cache explícito e a feature opt-in `github-http`.
  - `run`, `check`, `build` e `disasm` consomem o grafo misto com `--package-cache`, sem rede ou escrita no cache; provenance `path`/`github` e paths físicos das entries são preservados.
  - A descoberta local não carrega edges GitHub; conflitos, ciclos, capability widening, locks ausentes/stale e artifacts ausentes/corrompidos continuam fail-closed.
- **CLI — Execução Direta de Bytecode `.aibc` e Subcomando `disasm`**:
  - `aipo run file.aibc`: execução direta de módulos bytecode pré-compilados, com detecção automática pela extensão `.aibc`, desserialização via `BytecodeModule::from_bytes`, verificação estrutural pelo `BytecodeVerifier`, e execução na VM sem passar pelo pipeline frontend (lexer→parser→HIR→sema→IR→emitter). Erros de desserialização e verificação reportam falha de linguagem (exit 1); arquivos inexistentes reportam erro de uso (exit 2).
  - `aipo disasm <file.aipo|file.aibc>`: novo subcomando de desassembly, exibindo listagem legível de constantes, nomes, funções e instruções. Para arquivos `.aipo`, compila e usa `disassemble_with_source` com anotações `[line:col]`; para arquivos `.aibc`, usa `disassemble` sem anotações de código-fonte. Erros de compilação/desserialização são reportados normalmente.
  - `docs/reference/cli.md`: referência normativa atualizada com ambos os comandos, exemplos e estabilidade declarada desde Wave 4.
- **Bytecode e VM — Serialização Binária Nativa `.aibc` e Mapeamento de Linhas**:
  - `aipo-bytecode`: implementação da serialização e desserialização binária nativa e determinística (`BytecodeModule::write_to`, `read_from`, `to_bytes`, `from_bytes`) com codificação BigEndian para cabeçalho mágico (`AIBC`), versão 1, tabela de nomes internados, pool de constantes tipadas, layouts de structs, metadados de funções compiladas (com slots de upvalues e flag assíncrona), fluxo de bytes de instruções e mapa de spans de código fonte.
  - `aipo-bytecode`: mapeamento de linhas e colunas no desassemblador via `disassemble_with_source(module, source)`, anotando cada instrução com `[line:col]` a partir dos spans do módulo compilado.
- **VM — Otimização de Layout de Memória e Eliminação de Alocações Transitórias**:
  - `aipo-vm`: empacotamento em `Rc` das variantes pesadas `Value::Closure(Rc<ClosureData>)` e `Value::BoundMethod(Rc<BoundMethodData>)`, reduzindo o tamanho de `Value` de ~72/80 bytes para 48 bytes (redução de até 40% na pilha de avaliação, frames locais e coleções).
  - `aipo-vm`: eliminação da alocação de `Box<Value>` para o receptor (`receiver`) dentro de métodos ligados (`BoundMethodData`), unificando o armazenamento em um único nó de heap e tornando a clonagem de closures instantânea (sem clonar o vetor de upvalues).
- **Maturidade e Fechamento Integral do Frontend V1**:
  - **Teste de Tipo Anulável em `is` (`value is Type?`)**: parsing em `aipo-syntax`, variante `BinaryOp::IsNullable` no AST, instrução `CoreInst::TypeIsNullable` no Core IR, OpCode 58 `TypeIsNullable` no bytecode, execução nativa na VM Rust e no runtime JavaScript (`aipo-runtime.js`), avaliando o sujeito uma única vez e retornando `true` imediatamente se o valor for `none` ou delegando ao teste de tipo concreto caso contrário, com 100% de paridade diferencial verificada.
  - **Açúcar Sintático de Múltiplos Sujeitos em `is` (`a, b, c is Type[?]`)**: suporte canônico (`docs/canon/Aipo V1 — Language Reference:131`) no parser expandindo `a, b, c is T` em `((a is T and b is T) and c is T)`, com avaliação estrita da esquerda para a direita, curto-circuito em `and`, suporte a tipos anuláveis e isolamento contextual de vírgulas dentro de argumentos de chamada, literais de coleção e padrões de `match`.
  - **Diagnóstico Educativo Canônico para `is not`**: detecção no parser de tentativas de uso da sintaxe Python `is not`, gerando diagnóstico antecipado `AIPO_PARSE_UNEXPECTED_TOKEN` com sugestão explícita para uso da forma canônica `not value is Type`.
- **Maturidade da Stdlib e Backend — Módulos Canônicos `binary`, `time` (Tipos Puros e Parsers), `expect`/`testing`, e `log`**:
  - `binary`: operações completas de leitura e escrita big-endian e little-endian sobre `Bytes` (`read_*_be`, `read_*_le`, `write_*_be`, `write_*_le` para `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`), codificação e decodificação de varints LEB128 não-assinados (`read_varint`, `write_varint`) com validação de limite seguro (`MAX_SAFE_INT`), e fatiamento tolerante `slice(start, end)`.
  - `time`: introdução dos tipos puros de calendário civil e horários (`Date`, `TimeOfDay`, `DateTime`, construtores `time.date`, `time.time_of_day`, `time.date_time`, `time.duration`), cálculo de anos bissextos e dias desde o epoch Unix via algoritmo civil puro de Howard Hinnant (`days_since_epoch`), parsers ISO 8601 (`parse_date`, `parse_time`, `parse_iso`), formatadores `.to_iso()` e cálculo de segundos Unix via `.epoch_seconds()`.
  - `testing` & `expect`: suíte canônica de asserções puras (`expect.equal`, `expect.not_equal`, `expect.true`, `expect.false`, `expect.none`, `expect.some`, `expect.failure`, `expect.contains`, `expect.approx`) retornando `none` em caso de sucesso e `Failure` recuperável (Model B) em caso de divergência.
  - `log`: módulo de logging estruturado baseado em níveis (`trace`, `debug`, `info`, `warning`, `error`), com sink de log configurável e roteamento para o sink de `io` por padrão.
  - Runtime e VM: isenção das asserções de inspeção de falhas (`expect.failure`, `testing.failure`) da auto-propagação imediata de argumentos em chamadas de função, permitindo asserir falhas recuperáveis sem aborto prematuro.
  - Paridade diferencial VM↔JS: 26 testes diferenciais bit-a-bit passando 100% de forma determinística no Node.js e na VM Rust.
- **Maturidade da Stdlib e Backend — Módulos Canônicos `encoding`, `path`, `url` e `regex`**:
  - `encoding`: codificação e decodificação estrita de `base64_encode`, `base64_decode`, `base64url_encode`, `base64url_decode`, `hex_encode`, `hex_decode`, `utf8_encode`, `utf8_decode` com `Failure` recuperável (Model B) em entradas inválidas.
  - `path`: manipulação puramente lógica e normalização de caminhos (`join`, `normalize`, `is_absolute`, `basename`, `dirname`, `ext`) com suporte a múltiplos separadores (`/`, `\`).
  - `url`: modelo canônico WHATWG URL em `url.parse(text)` retornando dicionário estruturado com `href`, `origin`, `protocol`, `username`, `password`, `host`, `hostname`, `port`, `pathname`, `search`, `hash` ou `Failure` em URLs inválidas.
  - `regex`: motor regex em tempo linear (`regex.compile`, `regex.is_match`, `regex.replace`) retornando struct `Pattern` com métodos (`is_match`, `find`, `find_all`, `replace`, `split`) com 100% de paridade diferencial VM↔JS.
  - `collections`: alinhamento de vocabulário único eager/lazy em `List` (`first_or`, `last_or`, `find_index`, `take`, `skip`, `distinct`, `zip`, `chain`, `chunk`, `window`, `enumerate`).
  - `string`: operações canônicas human-facing em aglomerados de grafemas estendidos (`graphemes()`, `words()`, `lines()`, `casefold()`, `encode_utf8()`, e `Bytes.decode_utf8()`).
- **Maturidade da Stdlib e Backend — Operações Ávidas de Coleções e Métodos de Dicionário**: implementação de métodos ávidos de ordem superior em listas (`list.map(fn)`, `list.any(fn)`, `list.all(fn)`, `list.flat_map(fn)`, `list.reduce(init, fn)`) no despachador de métodos da VM (`aipo-vm/src/vm/call.rs`, `aipo-vm/src/vm/method.rs`) e no runtime JS (`aipo-runtime.js`), além de `dict.entries()` retornando pares `[[k, v]]`.
- **Maturidade da Stdlib e Backend — Extensões Canônicas do Módulo `math`**: trigonometria (`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`), funções logarítmicas/exponenciais (`log`, `log2`, `log10`, `exp`), além de `hypot`, `sign`, `rad`, `deg` em `aipo-stdlib/src/math.rs` e `aipo-runtime.js`, com validação estrita de domínio gerando `Failure` recuperável (Model B) em `log(<=0)` e `asin/acos(|x|>1)`.
- **Maturidade da Stdlib e Backend — Módulo `random` Portável (PRNG SplitMix64 Determinístico)**: implementação exata a nível de bits do PRNG SplitMix64 em `aipo-stdlib/src/random.rs` e `aipo-runtime.js` (`random.create(seed)`, `random.seed(seed)`, `int(min, max)`, `float()`, `bool()`, `choice(list)`, `shuffle(list)`), com suporte a despacho de métodos nativos em instâncias de `Struct` (`Rng`).
- **Maturidade da Stdlib e Backend — Módulo `json` Portável**: serializador e deserializador canônico em `aipo-stdlib/src/json.rs` e `aipo-runtime.js` (`json.parse` com rejeição estrita de chaves duplicadas como `Failure`, `json.stringify` com detecção de ciclo e formatação canônica de floats e inteiros), com 100% de paridade diferencial verificada.
- **Maturidade do Frontend — Lambdas Curtas `=>`**: suporte completo a lambdas de expressão concisas (`x => expr`, `_ => expr`, `() => expr`, `(a, b) => expr`, `(a, _) => expr`) no lexer (`TokenKind::FatArrow`), parser (com lookahead `is_lambda_ahead` semântico), HIR, IR, VM e `aipo-js`, com paridade diferencial 100% verificada no Node.js.
- **Maturidade do Frontend — Continuações Elididas de Comparações**: parsing e desaçucaramento canônico de cadeias com sujeito elidido (`val >= 0 and <= 100`, `val is Int and >= 0 and <= 100`), além de diagnóstico educativo amigável `AIPO_PARSE_UNEXPECTED_TOKEN` para comparações encadeadas diretas sem `and` (`1 < x < 10`).
- **Maturidade do Frontend — Otimização Zero-Alloc no Lexer**: eliminação de alocação de `Vec<(usize, char)>` no scan de arquivos inteiros; matching de keywords diretamente sobre fatias de bytes/string `&str` sem alocação intermediária; validação e parsing de literais numéricos em `&[u8]` sem cópias desnecessárias de string.

### Corrigido
- **Auditoria de IR, Host e Runtime — Semântica Canônica e Robustez de Fronteira**:
  - `aipo-ir` (navegação segura): `p?.name` e `svc?.call(args)` baixam para um teste de `none` com o receptor avaliado uma única vez; se o receptor for `none`, nem o acesso nem os argumentos executam e a expressão é `none` — em ambos os backends (canon §143; auditoria IR-5).
  - `aipo-ir`/`aipo-vm`/`aipo-js` (or_else lazy): `a or_else b` agora registra um handler de recuperação; o fallback só executa quando a esquerda termina em `Failure`, e um fallback que também falha propaga para o handler externo (auditoria IR-6/IR-7).
  - `aipo-ir` (falhas em controle de fluxo): condições de `if`/`elif`/`while`/`match`/`repeat`/`each` e condições de `if`-expressão propagam `Failure` recuperável via `PropagateFailure` antes do desvio, em vez de convertê-la em fault de tipo (auditoria IR-9).
  - `aipo-ir`/`aipo-vm`/`aipo-js` (iteração): `each` projeta bindings por modo (`IterAt`), então `each key[, value] in dict` itera chaves em ordem de inserção com o valor lido ao vivo, `each i, v in list` mantém índice/elemento e `each x in list` o elemento; guarda de mutação preservada em ambos os backends (auditoria IR-10).
  - `aipo-ir`/`aipo-vm`/`aipo-js` (unwind de loop): `break`, `continue` e `return` que saem de um `attempt`/`each` recolhem os handlers e guards abertos no corpo, eliminando handlers obsoletos e guards de iteração fantasma após o retorno (auditoria IR-8/IR-12). `IterGuard`/`IterGuardEnd` agora são balanceados também para iteráveis sem identidade (ranges, strings), com sentinela que nunca casa uma coleção real.
  - `aipo-ir`/`aipo-vm`/`aipo-js` (atribuição): um `Failure` no alvo ou no valor de `a.field = v`/`c[i] = v` propaga no ato (Model B) em vez de virar fault de tipo ou ficar na pilha; `SetField`/`SetIndex` deixam de despejar `none` na pilha do backend JS, eliminando o vazamento de operandos (auditoria IR-11/IR-18).
  - `aipo-ir` (await do): `await do` agora dessuga também `while`, `loop`, `repeat`, `attempt`, atribuições, `compound assign`, `return` e `fail`, e reconhece métodos assíncronos registrados como `Type.method` (auditoria IR-13).
  - `aipo-ir` (capturas, pipeline e contratos): corpos de closures anônimas não poluem mais a coleta de nomes livres do escopo externo; `a |> f` estaciona o argumento antes do callee; `return` sem valor sob contrato não-nulável emite `none` + checagem e falha como violação de contrato (auditoria IR-14/IR-16/IR-17); `a?.b = v` é rejeitado de forma consistente entre parser, sema e IR; alvos inválidos emitem `Fail` explícito; nomes de hidden locals não colidem mais com bindings do usuário; `each` sem nomes não lê a coleção.
  - `aipo-host` (fronteira): `HostValue::validate` re-checa `Int`/`Float` construídos diretamente nas variantes; nomes vazios/duplicados de tipos, valores, campos, parâmetros e operações de handle são rejeitados; profundidade de contratos limitada; parâmetro obrigatório após opcional e `subject` opcional rejeitados; `clear()` não queima gerações de slots já livres; mensagens de fault sem problemas renderizam sentença completa (auditoria N-1..N-9).
  - `aipo-runtime` (determinismo): registro duplicado de módulo é rejeitado em vez de sobrescrever silenciosamente, caminhos canônicos validados, dependência ausente reportada em ordem lexicográfica, `NativeRegistry` reporta colisões, ordena listagens e normaliza módulo vazio para o Prelude, `InitializationFailed` carrega `[AIPO_RT_MODULE_INIT_FAILED]` no `Display` e o fallback de ciclo do Kahn reporta os nós não resolvidos (auditoria N-1..N-11).
  - Testes: 8 testes diferenciais VM↔JS novos (`safe_navigation`, `or_else_lazy`, `condition_failure`, `each_dict`, `assign_pipeline`, `await_do_blocks`, `loop_unwind`), 6 testes unitários de lowering no `aipo-ir`, matriz de contrato para `return` vazio, validações de host, testes de runtime e remoção da corrida de arquivo temporário na suíte de conformance da CLI. Shim JS versionado em `1.2.0`.
- **Bytecode e VM — Correções Críticas de Segurança, Concorrência e Verificador**:
  - `aipo-vm` (Segurança Sandbox): travessia recursiva profunda em `publish_check` em `aipo-vm/src/host.rs` inspecionando `Value::Closure` (células de upvalues), `Value::BoundMethod` (receptor) e `Value::Sequence` (cache, pipeline e geradores), prevenindo escape de handles e capacidades não-publicáveis fora de seu escopo.
  - `aipo-vm` (Concorrência Assíncrona): detecção de ciclos de await transitivos estendida para abranger `WaitTarget::Join` e grupos estruturados de tarefas (`JoinKind::All` e `JoinKind::GroupWait`), detectando deadlocks circulares e emitindo `AIPO_RT_AWAIT_CYCLE`.
  - `aipo-vm` (Ordem Superior): métodos ávidos de lista (`filter`, `any`, `all`, `flat_map`) preservam o erro exato original (`VmError`) retornado pela função de callback em vez de substituir espuriamente por `StackUnderflow`; liberação garantida do guard de mutação em iteração ativa (`active_iterations`) no método `reduce` mesmo na ocorrência de falhas ou erros na função de callback.
  - `aipo-bytecode` (Verificador Estrutural): validação estrita de bounds de slots locais (`GetLocal`, `SetLocal`, `JumpIfSetLocal` limitados a `params + locals`), slots de upvalues (`GetUpvalue`, `SetUpvalue` limitados a `func.upvalues`) e integridade de contagem de campos em `BuildStruct` comparados ao layout declarado da struct.
  - `aipo-bytecode` (Emissor): substituição de fallback arbitrário para `Add` em `CoreInst::Binary` por erro de compilação explícito caso um operador de IR não seja reconhecido.
- **Maturidade do Frontend — Varredura e Correções Canônicas Integrais**:
  - `aipo-source`: cálculo de linha e coluna UTF-8 em `Source::location` corrigido com alinhamento seguro à fronteira de caractere (`is_char_boundary`), garantindo compatibilidade com MSRV 1.85.0 e prevenindo erros em offsets em código com caracteres multi-byte.
  - `aipo-lexer`: tratamento e rejeição canônica de literais numéricos com ponto sem fração seguidos de expoente (`1.e5`) e com sublinhados inválidos (`1._5`), emitindo `AIPO_LEX_INVALID_NUMBER`.
  - `aipo-syntax`: suporte e testes da forma condicional inline de valor (`if c then a else b`), preservação do modificador `async` em assinaturas de métodos de interface (`interface ... async fn ... end`).
  - `aipo-hir`: unificação estrita de spans no hook `invariant()` (`acc.span().merge(right.span())`) garantindo diagnósticos apontando exatamente para o bloco de expressão correto.
  - `aipo-sema`: validação profunda de contratos de interface em `satisfy` — verificação de presença, aridade de parâmetros explícitos (descontando `self`), mutabilidade de receptor (`self!`), modificador `async`, contratos de tipo de parâmetros e tipo de retorno (`AIPO_SEM_CONTRACT_VIOLATION_STATIC`); validação estática de reatribuição de campos declarados `fixed` em instâncias de struct conhecidas (`AIPO_SEM_FIXED_REASSIGN`); registro e checagem de aridade e contratos de métodos de `impl` e funções locais.
  - `aipo-testkit`: injeção de manifesto de pacote ESM (`package.json` com `{"type": "module"}`) em diretórios temporários de bundles JS gerados para execução com Node.js, eliminando warnings espúrios na saída de erro padrão no Node 22+.
- **Maturidade do Frontend — Correções Semânticas e Lowering**:
  - `Assign` e `CompoundAssign` analisam a expressão do lado direito como topo (`analyze_expr_top`), evitando falso positivo de `AIPO_SEM_AWAIT_IN_SUBEXPRESSION` em comandos de atribuição.
  - Verificação estrita de mutabilidade de caminhos com validação de receptor `self!` vs `self` em métodos e proibição de reatribuição de campos declarados `fixed` fora de `init` (`AIPO_SEM_FIXED_REASSIGN`).
  - Verificação de tarefas esquecidas (`AIPO_SEM_FORGOTTEN_TASK`) estendida para chamadas de métodos assíncronos em instâncias (`expr.method()`).
  - Acumulação de span com `merge` correto no lowering de hooks de invariante em `aipo-hir`.
  - Recuperação resiliente de erros em declarações no parser via `synchronize()` com garantia estrita de progresso e prevenção de estouro de pilha sob aninhamentos profundos de parênteses (`parse_paren_expr` e helpers de lambda `#[inline(never)]`).
- **Maturidade de IR, Host e Runtime — Auditoria e Correções de Corretude**:
  - `aipo-ir`: `CompoundAssign` em `Index` avalia `base`/`idx` uma única vez via hidden locals (`list[f()] += 1` sem duplo efeito colateral); targets inválidos emitem `Fail` explícito em vez de silêncio (backstop do `check_assignment_target` da sema); `FnDecl` registra o nome declarado em `seen` na coleta de livres.
  - `aipo-host`: `HostFault::Schema` mapeia para o novo `AIPO_RT_INVALID_SCHEMA` (antes `AIPO_PARSE_UNEXPECTED_TOKEN`); `HandleTable::clear()` repopula a free-list reaproveitando slots; namespace por categoria em `ModuleSchema::validate` documentado como intencional.
  - `aipo-runtime`: `InitializationFailed` mapeia para o novo `AIPO_RT_MODULE_INIT_FAILED` (antes `AIPO_RT_TYPE_MISMATCH`); `NativeRegistry` documentado como catálogo de introspecção/tooling (despacho real por nome em `aipo-vm/src/vm/call.rs`); `topological_init_order` com min-heap O(n log n).

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
- **Wave 4 — Adaptador Poppy e demo headless determinística (P03-G02)**: novo crate `aipo-poppy` implementando o perfil de referência do Poppy Game Engine sobre `aipo-host`. AHS canônico (`poppy_schema`) expondo tipos (`Vec2`, `Transform`), handles (`Entity`) e funções (`spawn`, `despawn`, `query`, `get_position`, `set_position`, `get_velocity`, `set_velocity`, `random_float`, `random_int`, `step`, `digest`); handles geracionais sem use-after-free (`AIPO_RT_STALE_HANDLE`); buffer de comandos (`CommandBuffer`) com mutações estruturais (spawn/despawn) deferidas a safe points; PRNG determinístico com semente (`PoppyRng`); simulação headless em taxa fixa (`Simulation`); adaptador de VM expondo o módulo `poppy` sob a árvore de capabilities `poppy.*` (`poppy.ecs`, `poppy.random`); fixture de integração com script real de jogo (`tests/headless_demo.rs`) comprovando digest idêntico entre execuções repetidas com a mesma semente, variação sob sementes distintas, fault em handle obsoleto e negação de capability (`AIPO_RT_CAPABILITY_DENIED`).
- **Wave 4 — escape de escopo imposto nos pontos de publicação (P03-G01)**: um binding criado dentro de um callback de host escopado não pode sair do escopo, e a regra é verificada onde o valor se torna alcançável no heap — `SetGlobal`, `Return`, `SetField`, `SetIndex`, `BuildList` e `BuildDict` —, cada um nomeando o ponto na mensagem de `AIPO_RT_SCOPE_ESCAPE`. Fechar um escopo libera os handles que ele cunhou pela própria tabela geracional (a geração avança, então nenhum deles volta a resolver memória do host) e os lembra à parte, para que a tentativa de publicação reporte o fault específico em vez de uma leitura obsoleta. A verificação é pulada enquanto nenhum escopo fechou, então um programa sem bindings de host (todos os das Waves 1–3) paga um booleano por ponto de publicação. Testes dirigem o pipeline real para cada um dos seis opcodes.
- **Wave 4 — módulo `time` atrás da capability `clock` (P03-G01)**: `time.now()` (wall) e `time.monotonic()` como `Duration` em segundos, sem fallback — sem relógio instalado a leitura é fault `AIPO_RT_CAPABILITY_DENIED` nomeando a capability, nunca um zero silencioso. `time::install_clock`/`revoke_clock` governam o serviço; o perfil da CLI instala `SystemClock` (então `aipo run` lê os relógios do SO) e um perfil determinístico instala seu próprio `ClockSource`, que é o que faz um replay repetir. Paridade VM↔JS implementada: o entry emitido instala o relógio do sistema e um host que precise de determinismo injeta sua fonte em `globalThis.__aipoClock` antes do bundle carregar (ou `null` para negar). A suíte diferencial passa a rodar sob um perfil determinístico único dos dois lados — sem ele a comparação de paridade não teria sentido. Conformance: programa 28.
- **Wave 4 — ABI de host (`aipo-host`, `P03-G01`)**: novo crate com o contrato entre host e script: **AHS** (`HostSchema`) descrevendo módulos, tipos, valores, handles, funções, parâmetros, contratos de retorno, mutabilidade, async, docs, capabilities e depreciação como dado, com validação que reporta todos os problemas de consistência de uma vez; **capabilities deny-by-default** onde o caminho *é* a árvore (`clock` cobre `clock.wall`/`clock.monotonic`, `poppy` cobre `poppy.*`) e um conjunto declarado é limite superior que a política só pode estreitar; **handles geracionais** que nunca usam estado liberado (liberar incrementa a geração, então o handle antigo resolve como obsoleto e nunca para o valor que reusa o slot, com aposentadoria do slot em vez de wrap); **valores de host** por cópia, com `Int` em ±(2^53−1) e `Float` finito checados na fronteira; e **faults** com código estável (`AIPO_RT_CAPABILITY_DENIED`, `AIPO_RT_STALE_HANDLE`, `AIPO_RT_SCOPE_ESCAPE`). Sem dependência de engine e sem referência/lifetime Rust atravessando a fronteira. Adaptador completo no VM (`aipo-vm/src/host.rs`) e testes de integração de pipeline (`tests/host_scope_escape.rs` e `tests/host_capability_and_handles.rs`).
- **Wave 3 encerrada (P02-G01, P02-G02, P02-G03 → DONE)**: os três slices da Wave 3 estão certificados e o gate de saída de `docs/waves/wave-3-async.md` está cumprido (programas 24–27 e diagnósticos 20–29 verdes na VM e no Node; `AIPO_RT_CANCELLED` e `AIPO_RT_AWAIT_CYCLE` como faults commitados; fmt/clippy/test/doc verdes; `unsafe_code = "forbid"` e MSRV 1.85 inalterados). Pendências registradas, não escondidas: `Task[T]` (ADP-006 §G) e `Date`/`TimeOfDay`/`DateTime` (ADP-006 §D, pacote `timezone` da Wave 6).
- **Wave 3 infra assíncrona: sintaxe `async fn`, `await do` e diagnósticos (P02-G03)**: `async fn` como protocolo de chamada em declarações de topo, locais, anônimas e métodos de `impl` (o receptor viaja como argumento 0 do corpo spawnado); `await do … end` como açúcar sequencial sobre `await`; diagnósticos estáticos `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`, `AIPO_SEM_FORGOTTEN_TASK` e `AIPO_SEM_NESTED_AWAIT_DO`; faults de runtime `AIPO_RT_AWAIT_CYCLE` e `AIPO_RT_CANCELLED` com paridade VM↔JS; endurecimento de literais numéricos com helper único em `aipo-lexer::number` (bases, separadores, limites de `i64`) usado pelo IR builder. Conformance: programas 24–27, diagnósticos 20–29.
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
- **`await` de literal só falhava em runtime (P02-G03)**: um `Task` só nasce de chamada `async fn` ou combinador, então `await <literal>` é provável antes da execução e passa a reportar `AIPO_SEM_CONTRACT_VIOLATION_STATIC` (ADP-006 §G) em vez de exigir execução.
- **Recovery de contrato paramétrico comia o `)` (P02-G03)**: o skip de `Task[T]`/`List[Int]` consumia o fechamento do chamador e cascateava um `AIPO_PARSE_UNEXPECTED_TOKEN` falso depois do `AIPO_SEM_PARAMETRIC_CONTRACT` real.
- **`async fn` de topo não era um item (P02-G03)**: `is_item_start` omitia `TokenKind::Async`, então uma declaração `async fn` no topo era parseada como statement local — `AIPO_SEM_FORGOTTEN_TASK` nunca disparava para ela.
- **`async fn` anônima travava o parser (P02-G03)**: `async fn(...) … end` como valor não avançava além de `fn`.
- **Laço infinito ao dirigir uma `Task` descartada (P02-G03)**: dirigir/aguardar uma task cujo desfecho já fora consumido reexecutava o corpo indefinidamente; o desfecho passa a ser absorvido no boundary e o ciclo de `await` faulta (`AIPO_RT_AWAIT_CYCLE`), nos dois backends.
- **Métodos `async fn` de `impl` não spawnavam (P02-G03)**: `register_struct_method` propagava o flag `async` apenas para funções livres.
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
- **Performance — decisão sobre cache de metadata de métodos**: o protótipo de cache por site/tipo foi medido no workload `fields`, registrou 199.998 hits e 2 misses, mas não trouxe ganho reproduzível sob o runner compartilhado; foi revertido, mantendo os caches de fields e frame base. Evidências em `docs/performance/cross-language.md`.
- **Performance — pool de nomes de métodos**: as variantes experimentais com `Rc<str>` e `Owned`/`Shared` não mostraram ganho reproduzível e foram revertidas; a API pública de `BoundMethodData` permanece com `String`. Evidências em `docs/performance/cross-language.md`.

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
