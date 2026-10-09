---
title: "Testes e diagnósticos"
description: "Verificar comportamentos usando o CLI."
---
# Testes e diagnósticos

**III · Construir** · Capítulo 18 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Verificar comportamentos usando o CLI.

## Entenda o conceito

`aipo check` analisa fonte sem executar. `aipo test` reúne testes descobertos pelo projeto. Uma mensagem de erro é uma pista sobre a fase que falhou.

## Experimente

```aipo
fn soma(a, b) {
    return a + b
}
io.println(soma(2, 3))
```





## Exercício

Teste o arquivo com `check` e consulte o guia de execução de testes.

**Critério de conclusão:** Anote comando, saída e versão do CLI para reproduzir problemas.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/18_small_statistics.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Falhas recuperáveis](/learn/17-falhas) · [Miniaplicação organizada →](/learn/19-projeto)
