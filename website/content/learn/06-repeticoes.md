---
title: "Repetições"
description: "Repetir ações com uma condição de parada."
---
# Repetições

**II · Fundamentos** · Capítulo 6 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Repetir ações com uma condição de parada.

## Entenda o conceito

`while` testa uma expressão a cada iteração. `each` percorre uma coleção. Um laço precisa de uma forma clara de terminar.

## Experimente

```aipo
var n = 0
while n < 3 {
    io.println(n)
    n += 1
}
each nome in ["Ana", "Rui"] {
    io.println(nome)
}
```





## Exercício

Mude o limite para 5 e conte as linhas.

**Critério de conclusão:** O contador para após atingir o limite; cada nome aparece uma vez.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/16_ranges_repeat_each.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Condições](/learn/05-condicoes) · [Listas →](/learn/07-listas)
