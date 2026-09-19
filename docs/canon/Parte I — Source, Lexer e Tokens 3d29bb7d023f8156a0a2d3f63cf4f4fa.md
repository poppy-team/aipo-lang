# Parte I — Source, Lexer e Tokens

<aside>
🔤

Esta parte cobre o caminho do arquivo `.aipo` até o fluxo de tokens, enquanto introduz gradualmente o primeiro subconjunto de **Odin** necessário para implementar o frontend.

</aside>

## Objetivos

- Entender Source e entrada UTF-8.
- Implementar `SourceSpan`.
- Implementar `TokenKind` e `Token`.
- Construir um lexer manual incrementalmente.
- Produzir diagnósticos iniciais.
- Aprender apenas o Odin necessário para cada passo.

## Material existente

O caderno de frontend já contém a teoria e decisões acumuladas até o início da implementação prática; ele passa a ser tratado como material desta Parte I.

[Capítulo 1 — Fundamentos do Frontend](Capítulo 1 — Fundamentos do Frontend 3d19bb7d023f81bdbf7fd7bb5e711315.md)

[Capítulo 2 — Primeiro Projeto Odin e Núcleo do Frontend](Capítulo 2 — Primeiro Projeto Odin e Núcleo do Fro 3d29bb7d023f818abb8fd8698aa6699f.md)

## Revisão normativa que orienta o lexer

A implementação da Parte I deve reconhecer a superfície consolidada aprovada em 2026-09-07. Em especial, a gramática futura usará `each`, `impl`, `invariant`, `fixed`, `when`, `attempt` e `failed`, e não deverá consolidar como V1 as formas históricas `for`, regras `where`/`@`, `fn Type.name(...)`, `case` ou `try` agrupado.

A decisão entre keyword global e keyword contextual para cada palavra será estudada no momento apropriado do lexer/parser.

Referência: [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md).