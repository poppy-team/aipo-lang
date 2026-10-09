---
title: "Invariantes e transações"
description: "Entender regras de integridade e rollback."
---
# Invariantes e transações

**IV · Avançado** · Capítulo 22 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Entender regras de integridade e rollback.

## Entenda o conceito

Invariantes protegem o estado em fronteiras suportadas. Blocos de recuperação podem reverter alterações de acordo com a semântica do runtime; não generalize a atomicidade sem consultar fixtures.

## Experimente

```aipo
struct Carteira {
    var saldo = 10
}
Carteira:invariant {
    self.saldo >= 0
}
var c = Carteira{}
attempt {
    c.saldo = -1
} failed err {
    io.println("Operação recusada")
}
```





## Exercício

Identifique qual regra deve permanecer válida.

**Critério de conclusão:** A validade do saldo é mais importante que esconder a falha.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/13_mutation_and_rollback.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Interfaces estruturais](/learn/21-interfaces) · [Tarefas e async →](/learn/23-concorrencia)
