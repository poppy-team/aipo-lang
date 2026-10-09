---
title: "Métodos e comportamento"
description: "Associar operações a uma estrutura."
---
# Métodos e comportamento

**III · Construir** · Capítulo 14 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Associar operações a uma estrutura.

## Entenda o conceito

Aipo favorece dados e operações explícitas em vez de uma hierarquia de classes. Métodos usam `Tipo:nome`; operações que mutam o receptor declaram `var self`.

## Experimente

```aipo
struct Contador {
    var valor = 0
}
Contador:incrementar(var self) {
    self.valor += 1
}
var c = Contador{}
c.incrementar()
io.println(c.valor)
```





## Exercício

Adicione um método `somar_dois`.

**Critério de conclusão:** Mutação não deve ficar oculta na assinatura.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/24_idiomatic_aipo_showcase.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Estruturas](/learn/13-estruturas) · [Closures →](/learn/15-closures)
