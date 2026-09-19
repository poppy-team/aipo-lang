# Interlúdio — Revisão da Sintaxe antes da Implementação

<aside>
🧭

**Lição:** antes de cristalizar uma sintaxe no lexer e no parser, revisar se cada construção ainda respeita o modelo mental e a filosofia da linguagem.

</aside>

## Contexto

Antes do primeiro lexer real, uma auditoria da Especificação Viva revelou histórico sintático acumulado e construções que haviam crescido além da simplicidade desejada. Em vez de implementar dívida de design, a superfície foi revisada.

## Mudanças aprovadas

- Regras por campo `where`/`@` foram substituídas por `fixed` + bloco `invariant` com expressões normais.
- `each` substituiu `for`, sem alias.
- `impl Type` + `self`/`self!` substituiu a grafia repetitiva `fn Type.name(receiver, ...)`, preservando funções + dados e sem criar OO clássico.
- `match` usa `when`.
- `attempt ... failed binding ... end` substituiu `try ... else ... end` para tratamento agrupado.
- Função que não produz valor deixou de retornar `none` implicitamente; ausência de resultado e valor `none` passam a ser conceitos distintos.

## Por que isso melhora a Aipo

- Menos mini-DSLs e menos formas equivalentes.
- Menor densidade visual em `struct`.
- Reuso dos operadores Boolean normais para invariants.
- Operações associadas agrupadas sem classe, herança ou método virtual.
- Palavras de controle mais descritivas (`each`, `attempt`, `failed`, `when`).
- `none` ganha significado mais preciso: ausência intencional é diferente de uma operação que simplesmente não produz resultado.

## Consequência para o frontend

O primeiro lexer continua começando por `let age = 20`. Novas keywords não serão implementadas antecipadamente; elas entram incrementalmente quando exemplos reais exigirem. A decisão sobre keywords contextuais será feita no momento em que o lexer/parser chegar a essas construções.

## Referência normativa

[Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md)