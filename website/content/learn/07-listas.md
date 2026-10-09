---
title: "Listas"
description: "Agrupar valores em ordem e acessá-los por índice."
---
# Listas

**II · Fundamentos** · Capítulo 7 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Agrupar valores em ordem e acessá-los por índice.

## Entenda o conceito

Listas são coleções ordenadas. O índice inicial é zero; índices inexistentes não devem ser tratados como valores normais.

## Experimente

```aipo
var frutas = ["maçã", "pera"]
frutas.add("uva")
io.println(frutas[0])
io.println(len(frutas))
```





## Exercício

Mostre o último elemento depois de adicionar um item.

**Critério de conclusão:** A lista passa a conter três elementos.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/08_lists_dicts_and_slices.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Repetições](/learn/06-repeticoes) · [Dicionários →](/learn/08-dicionarios)
