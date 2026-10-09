---
title: "Condições"
description: "Escolher ações usando expressões booleanas."
---
# Condições

**II · Fundamentos** · Capítulo 5 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Escolher ações usando expressões booleanas.

## Entenda o conceito

`if` executa seu bloco quando a condição é verdadeira. `elif` e `else` permitem alternativas; o escopo dos blocos é explícito.

## Experimente

```aipo
let idade = 17
if idade >= 18 {
    io.println("Maior")
} else {
    io.println("Menor")
}
```





## Exercício

Teste com idades 10, 18 e 50.

**Critério de conclusão:** Apenas uma alternativa é executada em cada caso.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/03_control_flow.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Texto e interpolação](/learn/04-strings) · [Repetições →](/learn/06-repeticoes)
