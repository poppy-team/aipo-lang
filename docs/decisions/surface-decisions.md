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

### B — ADTs (`enum`)

### C — Narrowing de tipo

## Bug corrigido (era pré-existente)

`docs/conformance/programs/17_unicode_nfc.aipo` falhava: o **folder de constantes do IR**
concatenava strings com `format!` sem normalizar, enquanto o `Add` da VM compõe NFC. Como
`aipo run` usa `optimize = true` por padrão, literais inline (`"e" + "\u{0301}"`) eram
dobrados sem normalizar. Corrigido em `crates/aipo-ir/src/opt.rs` (o folder normaliza
igual ao runtime), com teste de regressão `test_folds_string_concatenation_to_nfc`.
Suíte de conformidade **verde pela primeira vez** (3/3 `test_programs*`).