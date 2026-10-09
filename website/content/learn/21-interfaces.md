---
title: "Interfaces estruturais"
description: "Descrever contratos de comportamento."
---
# Interfaces estruturais

**IV · Avançado** · Capítulo 21 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Descrever contratos de comportamento.

## Entenda o conceito

Uma interface declara operações exigidas. Aipo busca conformidade estrutural; a diretiva `#!satisfies` pertence à sintaxe-alvo e deve ser conferida antes de uso corrente.

## Experimente

```aipo
interface Nomeavel {
    nome() -> String
}
struct Pessoa { texto }
Pessoa:nome() -> String {
    return self.texto
}
```





## Exercício

Enumere as operações que a estrutura deve oferecer.

**Critério de conclusão:** Não confunda uma diretiva-alvo com uma verificação executada.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/14_interfaces_and_satisfy.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Contratos de assinatura](/learn/20-contratos) · [Invariantes e transações →](/learn/22-invariantes)
