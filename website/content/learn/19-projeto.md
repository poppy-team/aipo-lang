---
title: "Miniaplicação organizada"
description: "Reunir coleções, funções e dados."
---
# Miniaplicação organizada

**III · Construir** · Capítulo 19 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Reunir coleções, funções e dados.

## Entenda o conceito

Projetos pequenos ajudam a integrar conceitos. Defina a entrada, o estado, a transformação e a saída antes de adicionar abstrações.

## Experimente

```aipo
struct Tarefa {
    titulo
    var concluida = false
}
var tarefas = []
tarefas.add(Tarefa{titulo: "Aprender Aipo"})
each tarefa in tarefas {
    io.println(tarefa.titulo)
}
```





## Exercício

Crie uma função para concluir a primeira tarefa.

**Critério de conclusão:** Uma mudança deve poder ser testada sem reescrever o sistema.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/24_idiomatic_aipo_showcase.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Testes e diagnósticos](/learn/18-testes) · [Contratos de assinatura →](/learn/20-contratos)
