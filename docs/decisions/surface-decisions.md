# Roteiro: decided · deferred · pending

Registro vivo das decisões de superfície da linguagem Aipo, para não perder contexto
entre conversas. **Opção rejeitada** permanece registrada aqui de propósito: uma conversa
futura pode reabri-la sem reconstruir o histórico.

## Decidido

### A.1 — `expr?` (propagação de `Failure`)

- **Sintaxe:** `let v = f(x)?` — posfixo, liga mais forte que todo operador binário.
- **Semântica:** avalia o operando; se o resultado é `Failure`, propaga para o `attempt`
  mais próximo (ou para o caller quando não há handler); caso contrário, entrega o valor.
- **Legal em qualquer posição de expressão.** Diferente de `await`, que é restrito a
  statement, inicializador e valor de retorno (`AIPO_SEM_AWAIT_IN_SUBEXPRESSION`).
- **Chain:** o IR baixa `PropagateFailure` para `CheckFailure` (`OpCode`), que é exatamente
  a checagem que a VM já executa em todo limite de statement. Nenhuma instrução nova de VM.
- **Conformidade:** `docs/conformance/programs/29_try_propagation_operator.{aipo,stdout}`.
- **Testes:** `crates/aipo-cli/tests/try_operator_tests.rs` (3 verdes).

### A.2 — `expr with { ... }` (atualização funcional de struct)

- **Sintaxe:** `let novo = p with { y: 99 }` — **Opção A**, palavra-chave `with` + chaves.
- **Semântica:** produz um **novo** valor; `p` permanece intacto. Campos ausentes no bloco
  são copiados do original.
- **Lei de Shannon:** o bloco lista **somente as mudanças**, então a ordem dos campos não
  importa e não há estado intermediário observável.
- **Legal em qualquer posição de expressão**, como `?`.
- **Reuso:** o bloco é a mesma lista de campos de `Construct`, o que reaproveita a
  validação de campos de struct já existente no sema.

#### Opção C — registrada, não escolhida

`Point(p, y: 99)` — sem palavra-chave nova; reusa `Construct` com o struct como primeiro
argumento.

- **Prós:** zero sintaxe nova no lexer; possivelmente o menor diff no compilador.
- **Contras:** repete o nome do struct a cada uso; muda a precedência de `Construct`
  (passa a aceitar um valor base antes do brace); pior legabilidade em aninhamento
  (`Point(a, x: Point(b, x: 1))`).
- **Quando reabrir:** se a prioridade for eliminar expansão de sintaxe a qualquer custo,
  ou se `with` sair do escopo e a atualização de struct virar caso raro o bastante para não
  justificar palavra-chave. Reabrir exige revisar a ambiguidade com `Construct` posicional.

#### Descartado sem registro

- **Opção B** (`p with y: 99`, sem chaves): as chaves delimitam o bloco de mudanças e
  mantêm a leitura localmente previsível. Só difere de A por ruído visual; sem ganho.

## Pendente

### A.3 — Destructuring e guards em `match`

- **Destructuring — Opção A** (`when { nome, idade }`): sem repetir o tipo, porque a
  extração segue a mesma garantia do acesso por ponto — um campo ausente é `TypeMismatch`
  em runtime, e a tipagem que C (narrowing) trouxer só torna o erro mais cedo. Opção B
  (`when Point { nome, idade }`) foi considerada e dispensada por ruído; quando os arms
  misturam structs diferentes, o fallthrough natural já seleciona o primeiro que casa.
- **Guards — Opção A** (`when pattern if guard then`): reusa `if`, zero palavra nova.
  Um guard `false` cai para o próximo arm, não para o `else` — é um filtro, não um desvio.
- **Bindings vivem no arm:** declarados no escopo do arm e visíveis no guard e no corpo;
  não vazam para siblings nem para fora do statement.
- **Guard é condição:** `check_condition_bool` aplica, mesma regra de `if`/`while`.
- **Implementação:** destructuring vira `Load target → GetField → Store` antes do corpo
  (bindings estabelecidos antes do guard, para o guard testar os campos); guard vira
  `JumpIfFalse` para o fail-jump do arm. Destructure emite `Jump` incondicional para o
  corpo (não pode falhar) — esse salto faltou no primeiro build e deixava o arm
  inalcançável.
- **Wasm:** diante de destructuring ou guards retorna `UnsupportedStmt` explícito, em vez
  de compilar errado em silêncio (mesmo padrão de `Try`, que cai no catch-all).
- **Spec canônica contradiz:** `Language Reference` (l. 929) e `Especificação Viva`
  (l. 1368) colocam guards e destructuring **fora da V1**. A.3 é extensão deliberada;
  os docs precisam de uma seção fechando a mudança antes do release.
- **Conformidade:** `docs/conformance/programs/31_match_destructure_and_guards.{aipo,stdout}`.
- **Testes:** `crates/aipo-cli/tests/match_destructure_tests.rs` (6 verdes: bind, guards
  aceita/rejeita, subconjunto, guard em value pattern, escopo do arm, guard não-Bool).

### B — ADTs (`enum`) — **decidido 2026-10-06, promovido a V1**

Contexto: `enum` estava no backlog pós-V1. Promovido porque a linguagem não tem
outra forma de expressar "isto é A **ou** B" com verificação de cobertura, e o
`match` sobre tipo fechado é o que garante exaustividade em compile time.

Decisão de superfície (canon em `SYNTAX.md` §17, norma em `adr-001` §14):

- Três formas de variante: `Nome`, `Nome { campo: T }`, `Nome(valor: T)`. Sem
  sintaxe nova de payload.
- Construção é o nome qualificado: `Estado.Desligado("x")`. Sem `new`, sem `::`.
- `match` nomeia tipo **e** variante: `when Estado.Ativo { desde }`. Reaproveita o
  destructure de A.3; a diferença é que o `{ ... }` liga variante + campos.
- **Exaustividade é obrigação:** `match` sem `else` e sem todas as variantes é erro.
- `enum` não traz conceito novo de pattern nem de binding: compõe com `fail`,
  `Task`, `Tipo:invariant` e `#!satisfies` como `struct`.

Mutabilidade em batch (mesmo item, decidido junto): batch exige `self` como primeiro
parâmetro da função de origem; `self` → método read-only, `var self` → mutável.
Sem token novo — `var` mantém o mesmo significado em toda a linguagem.

Representação em runtime (decidido 2026-10-07): **erasure para struct nomeada
`"Enum.Variante"`** como representação V1. Cada variante vira uma instância de struct
com esse nome de tipo; variantes tupla ganham um construtor sintetizado `E.V(params)`;
`match` despacha via opcode `IsVariant` (comparação de nome, com sufixo para
qualificação de módulo) e o payload é lido com `GetField` comum. Paridade total
VM↔JS pelo mesmo esquema; Wasm rejeita explicitamente (`UnsupportedStmt`,
pino em `crates/aipo-wasm/tests/match_tests.rs`). Uma tagged union real
(`Value::Enum` com discriminante) fica para o futuro se medição mostrar que o
despacho por string é gargalo — a troca é interna, sem mudar a superfície.
Validade estática garantida no sema: variante duplicada
(`AIPO_SEM_REDECLARED_IN_SCOPE`), variante desconhecida em `match` e em chamada
(`AIPO_SEM_UNKNOWN_NAME`), aridade de tupla e chamada de variante não-tupla
(`AIPO_SEM_ARITY_MISMATCH`), exaustividade sem `else`
(`AIPO_SEM_NON_EXHAUSTIVE_MATCH`). Fixture observável:
`docs/conformance/programs/32_enum_variants_and_matching.aipo`.

### C — Narrowing de tipo

## Decisões de sintaxe canônica (filosofia ND) — 2026-10-06

Contexto: Aipo é uma linguagem fácil de ler, escrever e manter, com prioridade para
pessoas neurodivergentes (TDAH, dislexia, sobrecarga cognitiva). Regras que guiam
tudo abaixo: **uma forma só para cada coisa**, palavras completas em vez de símbolos
densos, e erros que dizem o que fazer.

Status geral: **decidido, pendente de implementação**. O compilador ainda aceita as
formas antigas; os docs normativos (`SYNTAX.md`, Language Reference, Especificação
Viva) ainda as descrevem. A implementação remove o código morto e atualiza os docs
na mesma mudança — nunca um sem o outro.

### T1 — Blocos: só `{}` (`end` removido)

Toda abertura usa `{`, todo fechamento usa `}`. A keyword `end` sai do lexer.
Motivo: `end` triplo é ambíguo visualmente; chaves têm contorno claro e os editores
destacam pares. Já decidido em ADP-012; falta remover.

### T2 — Divisão inteira: só `//` e `//=`; `div`/`div=` removidos

`div` volta a ser identificador comum. Escolha **B** para a armadilha do `//`:
`//` continua divisão, mas o compilador ganha diagnóstico amigável — quando `//`
aparece onde nenhuma expressão válida segue, sugere `#` como comentário em vez de
emitir `DIV_ZERO` ou `UNKNOWN_NAME`.

### T3 — Tipos em struct: anotação opcional + `invariant()` para regras de valor

Campos aceitam tipo opcional (`id: Int`, `var status: String = "online"`).
`invariant()` **não** é a ferramenta para tipos: ele roda em runtime, depois da
construção e nas fronteiras mutáveis, então serve para regras de **valor**
(`self.id > 0`), não para promessas de tipo. Anotação = o que o autor promete
(permite checagem estática quando C/narrowing chegar); invariante = o que o valor
precisa satisfazer (sempre checado em runtime). Sem anotação, nada muda para quem
está prototipando — divulgação progressiva, sem punir exploração.

### T4 — Receptor: só `self` (imutável) e `var self` (mutável)

`self!` sai do parser (token `SelfMut` removido). Uma regra só: `var` = mutável,
em todo lugar. Já decidido em ADP-012; falta remover o código morto (parser ainda
aceita `self!` em `parse_params`).

### T5 — Parâmetros: `nome` (imutável) e `var nome` (mutável), com `: T` opcional

`nome!` sai do parser (o `!` é sutil demais para disléxicos e confunde com `not`).
Duas dimensões ortogonais: mutabilidade (`var` ou não) × tipo (`: T` ou nada).

### T6 — `then`: nunca em bloco, só no inline

Regra: tem `{}`? Sem `then`. Não tem `{}`? Com `then` (`if c then a else b`).
Remove a inconsistência atual (`then` opcional no bloco, obrigatório no inline).

### T7 — Construção qualificada `mod.Tipo{}` + diagnóstico amigável

`geo.Point{ x: 1 }` passa a valer. Minúscula inicial (`ponto{...}`) vira diagnóstico
("nomes de struct começam com maiúscula: você quis dizer `Ponto`?") em vez de erro
de parse genérico.

### Contagem de keywords (alvo)

Saem `end`, `div`/`div=`, `self!`: 46 → 43 (confirmar número exato na implementação,
quando os tokens forem removidos do `token.rs`).

### Docs que contradizem e precisam de atualização na implementação

- `SYNTAX.md` §1 e §12 ("duas terminações", tabela com `end`; `div` como keyword)
- Language Reference l.41 ("blocos terminam com `end`") e l.929 (match sem guards)
- Especificação Viva l.636–637 (`end` oficial) e l.1368 (destructuring/guards fora da V1)
- Interlúdio de Hooks (`self!` canônico), ADP-002 (`init(self!, ...)` nos exemplos)

### Em aberto

- **Tópico A** — keyword de função (`fn` vs `func` vs outras): em discussão.
- **Tópico B** — formato de erros/warnings: em discussão.

## Bug corrigido (era pré-existente)

`docs/conformance/programs/17_unicode_nfc.aipo` falhava: o **folder de constantes do IR**
concatenava strings com `format!` sem normalizar, enquanto o `Add` da VM compõe NFC. Como
`aipo run` usa `optimize = true` por padrão, literais inline (`"e" + "\u{0301}"`) eram
dobrados sem normalizar. Corrigido em `crates/aipo-ir/src/opt.rs` (o folder normaliza
igual ao runtime), com teste de regressão `test_folds_string_concatenation_to_nfc`.
Suíte de conformidade **verde pela primeira vez** (3/3 `test_programs*`).