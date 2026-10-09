---
title: "Transformando coleções"
description: "Utilizar funções para filtrar coleções."
---
# Transformando coleções

**III · Construir** · Capítulo 16 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Utilizar funções para filtrar coleções.

## Entenda o conceito

Métodos de alta ordem descrevem a transformação desejada. Considere legibilidade, ordem e custos de alocação antes de encadear várias operações.

## Experimente

```aipo
let numeros = [1, 2, 3, 4]
let pares = numeros.filter(fn (n) { return n % 2 == 0 })
each n in pares {
    io.println(n)
}
```





## Exercício

Filtre os valores maiores que dois.

**Critério de conclusão:** A condição de filtro devolve booleanos.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/24_idiomatic_aipo_showcase.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Closures](/learn/15-closures) · [Falhas recuperáveis →](/learn/17-falhas)
