---
title: Sintaxe do Aipo e divergências
---
# Sintaxe e diferenças com o compilador

O [ADR-001 de sintaxe](https://github.com/poppy-team/aipo-lang/blob/main/docs/decisions/adr-001-canonical-syntax.md) foi aceito em 6 de outubro de 2026. O [SYNTAX.md](https://github.com/poppy-team/aipo-lang/blob/main/SYNTAX.md) declara explicitamente que descreve uma **superfície-alvo ainda não integralmente implementada**. Não trate o texto do ADR como validação da CLI.

## Formas básicas

| Tema | Forma de referência |
| --- | --- |
| Variáveis | `let nome = valor`, `var nome = valor` |
| Funções livres | `fn nome(args) { ... }` |
| Condicionais | `if condição { ... } else { ... }` |
| Comentários | `# comentário` |
| Structs | `struct Nome { campos }` |
| Método | `Tipo:metodo(...)` |
| Mutabilidade do receptor | `var self` |
| String interpolada | `f"Valor: {n}"` |
| Iteração | `each item in lista { ... }` |
| Falha recuperável | `fail`, `attempt { ... } failed err { ... }` |

## Formas históricas e migração

| Antes ou coexistente no parser | Alvo aprovado |
| --- | --- |
| `end` | Fechamento com `}` |
| `div` / `div=` | `//` / `//=` |
| `self!` | `var self` |
| `impl Tipo { ... }` | `Tipo:metodo` |
| `satisfy` | Conformidade estrutural e diretivas quando aplicáveis |

O símbolo `//` na proposta representa divisão inteira, não comentário. A forma `#!satisfies` depende da gramática concreta.

## Limitações comprovadas no audit de outubro

O [journal da revisão](https://github.com/poppy-team/aipo-lang/blob/main/docs/journal/2026-10-05-syntax-drift.md) registrou limitações no parser/semântica, incluindo campos tipados de `struct`, nomes qualificados em contratos, reexportação de importados e validação de membros. O audit estava vinculado àquela revisão: revalide com a `main` atual antes de fechar uma divergência.

Para documentação de **comportamento atual**, consulte código/fixture e anote o commit. Para uma **mudança de linguagem**, abra decisão e adicione conformance. [Matriz de suporte](/reference/status).
