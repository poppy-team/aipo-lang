---
title: "Seu primeiro programa"
description: "Escrever um arquivo e executá-lo."
---
# Seu primeiro programa

**I · Começar** · Capítulo 1 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Escrever um arquivo e executá-lo.

## Entenda o conceito

Um programa Aipo é uma sequência de instruções. `io.println` escreve uma linha na saída padrão. Não é preciso declarar uma função principal para iniciar um script.

## Experimente

```aipo
io.println("Olá, Aipo!")
```


Salve como `hello.aipo` e execute, na raiz do repositório:

```bash
cargo run -q -p aipo-cli -- run hello.aipo
```




## Exercício

Mude a frase e execute outra vez.

**Critério de conclusão:** O terminal deve imprimir exatamente a nova frase.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/01_fizzbuzz.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Livro da Linguagem](/learn/) · [Valores e variáveis →](/learn/02-variaveis)
