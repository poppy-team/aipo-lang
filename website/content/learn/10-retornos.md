---
title: "Parâmetros e retornos"
description: "Produzir valores reutilizáveis usando `return`."
---
# Parâmetros e retornos

**II · Fundamentos** · Capítulo 10 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Produzir valores reutilizáveis usando `return`.

## Entenda o conceito

Uma função retorna um valor que pode alimentar outra expressão. Anotações de assinatura restringem argumentos e retorno; não são um convite a coerções implícitas.

## Experimente

```aipo
fn dobro(n: Int) -> Int {
    return n * 2
}
let resposta = dobro(21)
io.println(resposta)
```





## Exercício

Escreva `triplo` e utilize seu resultado.

**Critério de conclusão:** O dobro de 21 é 42.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/07_functions_defaults_named_args.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Primeiras funções](/learn/09-funcoes) · [Escopo e mutabilidade →](/learn/11-escopo)
