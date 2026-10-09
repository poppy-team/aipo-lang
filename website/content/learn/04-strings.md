---
title: "Texto e interpolação"
description: "Criar mensagens legíveis com strings."
---
# Texto e interpolação

**I · Começar** · Capítulo 4 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Criar mensagens legíveis com strings.

## Entenda o conceito

Strings usam aspas duplas. O prefixo `f` permite interpolar expressões. Comentários comuns começam com `#`; não use `//` como comentário.

## Experimente

```aipo
let nome = "Lia"
let nivel = 3
io.println(f"Jogador: {nome} | nível: {nivel}")
```





## Exercício

Acrescente uma variável `vidas` à mensagem.

**Critério de conclusão:** Os três valores aparecem sem concatenações extensas.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/09_strings_unicode_and_formatting.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Números e booleanos](/learn/03-numeros) · [Condições →](/learn/05-condicoes)
