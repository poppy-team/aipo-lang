---
title: "Falhas recuperáveis"
description: "Separar falhas esperadas de faults de programação."
---
# Falhas recuperáveis

**III · Construir** · Capítulo 17 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Separar falhas esperadas de faults de programação.

## Entenda o conceito

Aipo distingue falhas recuperáveis e faults. `attempt ... failed` trata falhas recuperáveis; use `or_else` quando um valor alternativo explícito for suficiente.

## Experimente

```aipo
fn dividir(a, b) {
    if b == 0 {
        fail "divisor zero"
    }
    return a / b
}
attempt {
    io.println(dividir(6, 0))
} failed erro {
    io.println("Não foi possível dividir")
}
```





## Exercício

Execute com divisor zero e depois divisor válido.

**Critério de conclusão:** A recuperação não deve esconder faults fatais.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/11_failures_or_else_attempt.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Transformando coleções](/learn/16-transformacoes) · [Testes e diagnósticos →](/learn/18-testes)
