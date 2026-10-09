---
title: "VM, JavaScript e Wasm"
description: "Escolher um backend pela cobertura demonstrada."
---
# VM, JavaScript e Wasm

**IV · Avançado** · Capítulo 25 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Escolher um backend pela cobertura demonstrada.

## Entenda o conceito

Stack VM é o caminho de referência. Register VM permanece experimental; JavaScript e Wasm possuem suportes e limites que devem ser medidos separadamente.

## Experimente

```bash
aipo run app.aipo
aipo check app.aipo
aipo build app.aipo --target js --out dist/
```



Este bloco é um comando de terminal; não salve como arquivo `.aipo`.



## Exercício

Compare as opções com `aipo --help` do binário local.

**Critério de conclusão:** O bloco contém comandos de terminal, não fonte Aipo.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/docs/reference/cli.md)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Pacotes e lockfiles](/learn/24-pacotes) · [Embedding e host APIs →](/learn/26-embedding)
