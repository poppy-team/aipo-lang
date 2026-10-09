---
title: "Números e booleanos"
description: "Realizar cálculos e testar condições."
---
# Números e booleanos

**I · Começar** · Capítulo 3 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Realizar cálculos e testar condições.

## Entenda o conceito

Números inteiros e de ponto flutuante possuem regras específicas. Uma comparação devolve `true` ou `false`. Não presuma coerção automática de strings.

## Experimente

```aipo
let preco = 120
let desconto = 20
let total = preco - desconto
let permitido = total <= 100
io.println(total)
io.println(permitido)
```





## Exercício

Altere o desconto para 15 e explique o resultado.

**Critério de conclusão:** O valor passa a ser 105 e a condição é falsa.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/20_safe_numeric_boundaries.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Valores e variáveis](/learn/02-variaveis) · [Texto e interpolação →](/learn/04-strings)
