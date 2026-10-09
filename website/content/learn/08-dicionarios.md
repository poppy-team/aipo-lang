---
title: "Dicionários"
description: "Consultar valores a partir de chaves."
---
# Dicionários

**II · Fundamentos** · Capítulo 8 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Consultar valores a partir de chaves.

## Entenda o conceito

Um dicionário associa chaves e valores. Para procurar uma chave use `get(chave)`; a API atual não aceita argumento extra de valor padrão.

## Experimente

```aipo
let pessoa = {"nome": "Ana", "nivel": 2}
io.println(pessoa.get("nome"))
io.println(pessoa.get("ausente") == none)
```





## Exercício

Adicione a chave `ativo` e consulte o valor.

**Critério de conclusão:** O caso de uma chave ausente fica explícito.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/08_lists_dicts_and_slices.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Listas](/learn/07-listas) · [Primeiras funções →](/learn/09-funcoes)
