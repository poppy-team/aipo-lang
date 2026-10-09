---
title: "Valores e variáveis"
description: "Distinguir valores imutáveis e variáveis reatribuíveis."
---
# Valores e variáveis

**I · Começar** · Capítulo 2 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Distinguir valores imutáveis e variáveis reatribuíveis.

## Entenda o conceito

`let` cria um vínculo imutável; `var` permite reatribuir. Use `let` quando o valor não muda. Uma variável não transforma automaticamente valores de outros tipos.

## Experimente

```aipo
let nome = "Ana"
var pontos = 10
pontos = pontos + 5
io.println(nome)
io.println(pontos)
```





## Exercício

Mude `var pontos` para `let pontos` e compare os diagnósticos.

**Critério de conclusão:** Uma tentativa de reatribuição imutável deve ser identificada.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/06_variables_and_values.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Seu primeiro programa](/learn/01-primeiro-programa) · [Números e booleanos →](/learn/03-numeros)
