# ADP-001 — Byte, `Bytes` e tipos como valores no Prelude V1

**Status:** resolved — Q1 e Q2 resolvidas por `P00-G10`; Q3 (limites invertidos de `clamp`), Q4
(saturação de slice) e Q5 (NFC nas fronteiras de construção) fechadas por `P00-G15`
**Related:** `docs/evidence/P00-G10-backend-completion.md`,
`docs/evidence/P00-G15-static-contracts-and-interface-conformance.md`,
`docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md`
**Origem:** Slice S9 (`P00-G09`), `crates/aipo-stdlib`, `crates/aipo-vm`
**Autoridade:** subordinada a `docs/canon/Aipo — Stdlib V1 Canônica e Contratos de Portabilidade…`,
`docs/canon/Aipo V1 — Language Reference…` e `docs/canon/Governança de Design e Evolução da Aipo…`
**Complementa:** `docs/stdlib/mvp-subset.md`

A política de não invenção (`prumo.json` → `risk`, `docs/language/authority-map.md`)
exige que toda decisão semântica material sem resposta em canon vire pergunta aberta. Este
documento reúne as que o slice S9 encontrou. Nenhuma delas bloqueia o MVP: nas partes não
ambíguas o comportamento implementado está documentado em `docs/stdlib/mvp-subset.md`.

## Q1 — `Byte` não possui kind de valor próprio no runtime

**Problema.** Canon trata `Byte` como tipo fundamental (inteiro compacto `0..=255`) e define
`Byte(value)` como conversão explícita e verificada: sem wraparound e sem saturação
silenciosa. O modelo de valores da VM (`aipo-vm::Value`) não possui variante `Byte`, e
nenhuma camada anterior (lexer, AST, HIR, IR, bytecode) conhece o tipo.

**Estado atual.** `Byte(value)` valida a faixa `0..=255` a partir de `Int`, `Float` integral
ou `String` e devolve o valor pela representação inteira compartilhada. Fora da faixa
produz `Failure` recuperável. Isso preserva a verificação obrigatória de canon e não
introduz wraparound, mas `Byte` e `Int` são hoje indistinguíveis em runtime.

**Pergunta.** Quando `Value::Byte(u8)` (ou um tipo gerenciado equivalente) entra, e o que
isso implica em: `type_name()` para diagnósticos, igualdade (`Byte(1) == 1`?), promoção
`Byte -> Int -> Float` nas operações aritméticas, literais `Byte` no lexer/AST e no pool de
constantes do bytecode?

**Fora de escopo deste slice.** Introduzir a variante toca S2/S6/S7 e não pertence a S9.

**Resolução (`P00-G10`).** `Value::Byte(u8)` existe (junto de `Bytes` e `Type`), com
`type_name()` próprio, igualdade estrutural com `Int`/`Float` (`Byte(1) == 1`), promoção
`Byte -> Int -> Float` nas operações (`Value::widened`/`Value::needs_widening`) e literais
admitidos pelas conversões. `Int` e `Byte` deixam de ser indistinguíveis em runtime.

## Q2 — Tipos como valores de Prelude (`List`, `Dict`, `Bytes`)

**Problema.** Prelude V1 (canon) lista `Int`, `Float`, `Byte`, `String`, `List`, `Dict`,
`Bytes` entre os nomes disponíveis sem import. `Int`/`Float`/`Byte`/`String` têm forma
chamável definida por canon (conversão). `List`, `Dict` e `Bytes` são usados como nomes de
tipo em contratos, mas canon não define nenhuma operação chamável para eles, e o runtime
não possui kind de *type value*.

**Estado atual.** `Int`, `Float`, `Byte` e `String` são registrados como globais nativos
chamáveis. `List`, `Dict` e `Bytes` **não** são registrados: registrá-los exigiria inventar
semântica (o que `List(x)` faria?) ou inventar um kind de valor de tipo.

**Pergunta.** Qual é o modelo de *type values* da V1: introspecção (`T.name`, `x is T` já
existe via sema), construção genérica (`List(x)`), ou tipos continuam sendo exclusivamente
notação de contrato sem representação em runtime? E `Bytes`, que não existe como tipo de
runtime em nenhuma camada, entra em qual slice?

**Resolução (`P00-G10`).** Tipos como valores existem: `Value::Type(TypeTag)` com `TypeTag`
enumerando `none`, `Bool`, `Int`, `Float`, `Byte`, `String`, `List`, `Dict`, `Bytes` e `Range`.
Todos os nomes estão vinculados no Prelude com identidade, `name()` canônico e igualdade
estrutural; `Int`/`Float`/`Byte`/`String` aceitam chamada (conversão), enquanto `List`/`Dict`/
`Bytes` são **apenas** type values — sem chamada em V1 (construção genérica não foi inventada).
A forma canônica de construir `Bytes` permanece aberta e é a lacuna G4 de ADP-002.

## Q3 — `clamp` com limites invertidos

**Problema.** Canon define `min`, `max` e `clamp` como utilitários numéricos e exige
promoção `Int -> Float` em operações mistas, mas não define `clamp(value, min, max)` com
`min > max`.

**Estado atual.** Bounds invertidos produzem `Failure` recuperável com mensagem explícita,
em coerência com o Modelo B (erro de domínio recuperável) e sem escolher silenciosamente
entre "retorna `min`" e "retorna `max`".

**Pergunta.** `Failure` é o canal correto, ou `clamp` invertido é erro de programação
(fault)? Existe variante canônica (`clamp_or`)?

**Resolução (`P00-G15`).** `Failure` recuperável, e a resposta vem do critério que o próprio
canon usa para separar os dois canais: a Language Reference §16 separa "falhas recuperáveis" de
"runtime faults de programação", e canonical classifica **valor fora de uma faixa** como
`Failure` — `Int(value)`/`Float(value)`/`Byte(value)` com valor fora do intervalo produzem
`Failure` "permitindo recuperação com `or_else`" (Language Reference §4), com "nunca wraparound ou
saturação silenciosa"; já **índice fora da faixa** é fault (indexação exata) e leitura/escrita fora
da área válida de `Bytes` é "fault de programação". Limites de `clamp` são *valores*, não índices
estruturais, então o canal coerente é `Failure`: `math.clamp(5, 3, 0) or_else -1` recupera, e um
`Failure` não tratado encerra o programa como qualquer falha recuperável. Nenhuma variante
(`clamp_or`) foi inventada: canon não a define e a forma `or_else` já é o fallback canônico.
Certificação: `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` e
`docs/conformance/diagnostics/19_runtime_clamp_inverted_bounds.aipo`.

## Q4 — `string.slice` fora da faixa

**Problema.** Canon exige runtime fault de indexação para `text[i]` fora dos limites, mas
não fixa o comportamento de `slice` com limites fora da faixa.

**Estado atual.** Limites são normalizados (índices negativos contam do fim) e saturados
aos limites da string; `start >= end` devolve `""`.

**Pergunta.** Slicing satura (implementado) ou falha como indexação? Se saturar, isso é
`String`-only ou também vale para `List[start..end]`?

**Resolução (`P00-G15`) — perguntado e respondido por canon.** A pergunta já estava respondida
para `List`: "Slices fora da faixa são tolerantes/clamped, ao contrário de índices exatos"
(`Aipo Language — Especificação Viva`, seção `List` — modelo fundamental decidido), junto de
"Slicing usa `list[start..end]`, com limite final exclusivo, limites opcionais e índices
negativos". O slice continuar criando valor novo e a indexação exata continuar sendo fault são
regras explícitas do mesmo trecho. A decisão registrada aqui é que **a regra de slice tolerante é
da categoria de slice, não de um tipo**: vale igualmente para `List`, `String` e `Bytes`, e
`string.slice(start, end)` segue a mesma normalização/saturação porque é a forma de método do
mesmo recorte. `start >= end` produz vazio em todos eles.
Certificação: `docs/conformance/programs/18_tolerant_slices_and_clamp.aipo` (limites acima do fim,
abaixo do início e invertidos em `List`, `String` e `Bytes`), complementando
`docs/conformance/programs/09_slicing.aipo` (slices dentro da faixa) e
`docs/conformance/diagnostics/07_runtime_index_out_of_range.aipo` (índice exato é fault).

## Q5 — NFC nas fronteiras de construção

**Problema.** Canon estabelece que `String` é UTF-8 normalizada em NFC nas fronteiras de
construção/decodificação, e que o resultado de `reverse()` obedece às invariantes de
`String` (Unicode válido, UTF-8, NFC).

**Estado atual.** `string.reverse` re-normaliza o resultado para NFC (única exigência
explícita de canon sobre o resultado de uma operação) usando `unicode-normalization`.
Demais operações (`lower`, `upper`, `replace`, `join`, `convert_string`, literais) não
normalizam, porque a normalização global é responsabilidade das fronteiras de construção
do modelo de `String` — ainda não implementada em nenhuma camada.

**Pergunta.** Onde a normalização NFC é aplicada de fato: no lexer (S2), na construção de
`String` (VM) ou em cada operação que pode quebrá-la? Qual slice assume isso?

**Resolução (`P00-G15`) — nas fronteiras, em todas as camadas que constroem `String`.** O texto de
canon que a pergunta cita ("normalizada automaticamente ... antes de ser exposta ao programa")
não nomeia uma camada, então a decisão é a menor que satisfaz o invariante sem uma varredura
global: normalizar **onde uma `String` é construída ou onde uma operação pode introduzir
desnormalização**. Ficaram aplicadas as três fronteiras reais de introdução:

1. **Literal de código-fonte** — o lexer decodifica escapes e normaliza o conteúdo antes de
   criar o token (`crates/aipo-lexer`), o que cobre literais comuns, multi-linha, `f"..."` e
   strings de módulo, e também os nomes de identificador.
2. **Conversão e concatenação** — `String(value)` normaliza na conversão, e `+` normaliza a
   concatenação, que é a operação por onde uma base pode encontrar uma marca combinante
   (interpolação `f"..."` é lowered para `+`, então herda a regra).
3. **Operações da stdlib que podem juntar base + marca** — `lower`, `upper`, `capitalize`,
   `replace`, `join`, `format` e `reverse`. Operações puramente substring (`slice`, `split`,
   `trim`) preservam o invariante de graça: remover caracteres nunca torna adjacentes dois
   caracteres que não eram.

`==` continua sendo igualdade estrutural sobre o valor guardado — canon diz que NFC é invariante
da `String`, não operação executada em `==`. Nada foi normalizado no caminho de leitura do
bytecode (pool de constantes): normalizar ali seria custo por acesso para um invariante já
garantido na única origem de literais. Isso fica explicitado porque é a escolha que pode precisar
revisão se uma futura fronteira de entrada (leitura de texto do host, `bytes.decode()`) entrar no
recorte MVP — hoje `io` não possui leitura de entrada e `bytes.decode()` está fora do MVP.
Certificação: `docs/conformance/programs/17_unicode_nfc.aipo` (literal com escape, concatenação,
interpolação, `join`, `replace`, `format`, `upper`, `reverse` e `capitalize`), mais os testes
`test_string_literals_are_nfc_normalized` (`crates/aipo-lexer/src/lib.rs`),
`test_string_operations_preserve_nfc` (`crates/aipo-stdlib/tests/stdlib_tests.rs`) e
`test_string_concatenation_preserves_nfc` (`crates/aipo-vm/tests/data_and_errors.rs`).

## Decisão final

Fechada. Q1 (kind `Byte`) e Q2 (type values) foram resolvidas por `P00-G10`; Q3 (limites
invertidos de `clamp`), Q4 (saturação de slice) e Q5 (NFC nas fronteiras de construção) foram
fechadas por `P00-G15`, com a fonte no canon citada em cada seção e a decisão registrada em
`docs/stdlib/mvp-subset.md`. A lacuna de construção de `Bytes` (parte de Q2) foi reafirmada como
critério de gate em `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` e fechada por
`P00-G13`. Nenhuma semântica foi inventada: onde canon não decide (`clamp` invertido), a decisão é
derivada do critério explícito do próprio canon para separar `Failure` de fault.
