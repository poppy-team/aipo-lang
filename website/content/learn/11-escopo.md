---
title: "Escopo e mutabilidade"
description: "Identificar onde cada variável pode ser utilizada."
---
# Escopo e mutabilidade

**II · Fundamentos** · Capítulo 11 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Identificar onde cada variável pode ser utilizada.

## Entenda o conceito

Nomes locais não devem escapar do bloco em que foram definidos. Prefira funções pequenas, dados explícitos e imutabilidade por padrão.

## Experimente

```aipo
fn somar(a, b) {
    let resultado = a + b
    return resultado
}
io.println(somar(3, 4))
```





## Exercício

Tente ler `resultado` fora da função.

**Critério de conclusão:** A tentativa fora do escopo deve produzir diagnóstico.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/21_closure_state.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Parâmetros e retornos](/learn/10-retornos) · [Módulos e arquivos →](/learn/12-modulos)
