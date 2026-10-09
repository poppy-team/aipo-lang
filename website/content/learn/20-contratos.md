---
title: "Contratos de assinatura"
description: "Explicitar requisitos de argumentos e retorno."
---
# Contratos de assinatura

**IV · Avançado** · Capítulo 20 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Explicitar requisitos de argumentos e retorno.

## Entenda o conceito

Contratos opcionais em assinaturas tornam interfaces mais claras. Eles não implicam que toda expressão tenha inferência estática ou que o runtime dispense validações.

## Experimente

```aipo
fn quadrado(n: Int) -> Int {
    return n * n
}
io.println(quadrado(5))
```





## Exercício

Passe um argumento incompatível e compare `check` e `run`.

**Critério de conclusão:** Observe em qual fase a incompatibilidade aparece.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/03_contracts_and_interfaces.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Miniaplicação organizada](/learn/19-projeto) · [Interfaces estruturais →](/learn/21-interfaces)
