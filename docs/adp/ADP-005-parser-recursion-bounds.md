# ADP-005 — Limites de Recursão do Parser (Limites de Aninhamento)

**Status:** aceito (implementado com este gauntlet; os limites são bounds de robustez)
**Relacionado:** `AIPO_PARSE_NESTING_TOO_DEEP` (catálogo de diagnósticos), `crates/aipo-syntax`
(guardas de profundidade + garantia de progresso), `crates/aipo-cli/tests/resource.rs`,
`docs/evidence/P01-G02-*.md`
**Autoridade:** subordinada à Language Reference (que não define nenhum requisito de
aninhamento) e à política de não invenção

## Fatos verificados (medidos, não presumidos)

- Antes desta mudança, um aninhamento hostil porém bem formado abortava o processo do host
  (`SIGABRT`, exit 134): `((((…))))` acima de ~500–800 níveis e `if` aninhado acima de
  ~300–600 níveis em uma main stack de 8 MiB. Um abort nunca é comportamento canônico: a
  estratégia de testes já exige que nenhum panic do Rust (e, por extensão, nenhum crash do
  host) escape como erro de usuário do Aipo.
- O custo medido de frame por nível é de ~10–27 KiB, dependendo da construção; portanto,
  qualquer limite precisa ficar bem abaixo de `2 MiB / 27 KiB ≈ 74` níveis para valer também
  em threads de teste de 2 MiB.
- O aninhamento mais profundo no corpus de conformidade tem um único dígito.
- Um segundo defeito latente surgiu da mesma investigação: os loops de corpo
  (`while !terminator { parse_stmt … }`) assumem que toda falha de parse consome entrada,
  de modo que *qualquer* falha que não consuma trava esses loops para sempre. A primeira
  versão da guarda de profundidade disparou exatamente esse caminho.
- Uma terceira forma escapou por completo das guardas dentro do parse: árvores construídas
  de forma iterativa, mas profundas (uma cadeia `1 + 1 + …` de 5000 termos, um `f"…"` com
  500 placeholders), que abortam os walkers recursivos posteriores (HIR lowering e adiante).
  A correção é um post-pass iterativo de profundidade da AST em `aipo_syntax::parse`
  (`depth.rs`, exaustivo sobre toda forma recursiva da AST, de modo que uma nova forma seja
  um erro de compilação, não uma brecha).

## Decisões (todas registradas aqui para que nada seja inventado silenciosamente)

1. **Limites:** aninhamento de expressões 128, aninhamento de blocos 64, profundidade da AST
   128. Todos são >15× o aninhamento mais profundo do corpus e mantêm a stack no pior caso
   em torno de ~1–2 MiB — dentro da menor stack de host suportada, com margem. Esses números
   são bounds de robustez da implementação, **não** semântica da linguagem: todo programa
   abaixo deles faz parse exatamente como antes (provado pela suíte de testes inalterada e
   pelos snapshots do corpus).
2. **Sinal:** um novo código de diagnóstico, `AIPO_PARSE_NESTING_TOO_DEEP`
   (severidade `error`), apontando para o token ofensor. Reutilizar
   `AIPO_PARSE_UNEXPECTED_TOKEN` descreveria mal o problema e prejudicaria o rubric de
   acessibilidade dos diagnósticos; um código dedicado é o sinal honesto.
3. **Controle de cascata:** após o primeiro overflow, o parser levanta uma flag de abort —
   o resto do arquivo é ignorado silenciosamente, em vez de emitir um erro por token
   restante (um arquivo com 600 níveis de profundidade produzia antes ~1000 erros
   subsequentes).
4. **Garantia de progresso:** uma falha de `parse_stmt` que não consumiu nada avança um
   token. Comprovadamente, esse caminho nunca dispara em entradas que já terminavam
   (elas sempre faziam progresso, caso contrário já travariam), de modo que o comportamento
   de recuperação observável ali permanece inalterado por construção.

## Não-objetivos

- Nenhuma sintaxe geral de limite de recursão, nenhuma flag de limite configurável, nenhuma
  afirmação sobre o que "deveria" aninhar profundamente. Se o canon algum dia exigir
  aninhamento mais profundo, o limite se move com evidência, não por edições que o
  contornem.
- A recursão do lowering de HIR/sema foi medida (um `if` de 300 níveis passa pelo lowering
  sem problemas), mas não tem guarda separada; o limite do parser restringe tudo o que vem
  depois. Se uma construção futura recursar fora do parser, ela ganha sua própria entrada
  de ADP.
