---
title: "Tarefas e async"
description: "Distinguir suspender uma tarefa e executar em paralelo."
---
# Tarefas e async

**IV · Avançado** · Capítulo 23 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Distinguir suspender uma tarefa e executar em paralelo.

## Entenda o conceito

O scheduler Aipo é cooperativo. `async fn`, `await do` e combinadores do módulo `task` seguem contratos próprios, não uma promessa de paralelismo em threads.

## Experimente

```aipo
async fn valor() {
    return 42
}
let tarefa = valor()
await do {
    io.println(tarefa)
}
```





## Exercício

Investigue cancelamento, término e propagação de falhas.

**Critério de conclusão:** Consulte o corpus específico de async; alguns backends diferem.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/docs/conformance/README.md)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Invariantes e transações](/learn/22-invariantes) · [Pacotes e lockfiles →](/learn/24-pacotes)
