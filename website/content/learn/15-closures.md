---
title: "Closures"
description: "Entender funções armazenadas como valores."
---
# Closures

**III · Construir** · Capítulo 15 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Entender funções armazenadas como valores.

## Entenda o conceito

Uma função anônima pode capturar valores do contexto. Isso é útil para callbacks e transformações, mas torna importante raciocinar sobre captura e tempo de vida.

## Experimente

```aipo
let fator = 3
let multiplicar = fn (n) {
    return n * fator
}
io.println(multiplicar(4))
```





## Exercício

Experimente alterar o valor capturado.

**Critério de conclusão:** Descreva de onde `fator` é obtido.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/21_closure_state.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Métodos e comportamento](/learn/14-metodos) · [Transformando coleções →](/learn/16-transformacoes)
