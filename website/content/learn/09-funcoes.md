---
title: "Primeiras funções"
description: "Reutilizar operações sem copiar código."
---
# Primeiras funções

**II · Fundamentos** · Capítulo 9 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Reutilizar operações sem copiar código.

## Entenda o conceito

Defina funções livres com `fn`. Nomeie funções pela operação que fazem. Os parâmetros recebem valores do chamador.

## Experimente

```aipo
fn saudar(nome) {
    io.println(f"Olá, {nome}!")
}
saudar("Ana")
saudar("Rui")
```





## Exercício

Escreva uma função para imprimir o dobro de um número.

**Critério de conclusão:** Cada chamada produz o resultado de seu argumento.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/02_local_functions.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Dicionários](/learn/08-dicionarios) · [Parâmetros e retornos →](/learn/10-retornos)
