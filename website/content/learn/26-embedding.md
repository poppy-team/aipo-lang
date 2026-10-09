---
title: "Embedding e host APIs"
description: "Compreender a relação entre um host e um script."
---
# Embedding e host APIs

**V · Integrações** · Capítulo 26 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Compreender a relação entre um host e um script.

## Entenda o conceito

O hospedeiro governa capacidades, funções exportadas, handles e limites. AHS descreve uma superfície: não instala implementações nem concede privilégios.

## Experimente

```bash
aipo run script.aipo --ahs host.json
```



Este bloco é um comando de terminal; não salve como arquivo `.aipo`.



## Exercício

Separe o que o arquivo AHS declara do que o host efetivamente implementa.

**Critério de conclusão:** Perfil shell mínimo ainda depende de decisões e validações.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/crates/aipo-host/README.md)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← VM, JavaScript e Wasm](/learn/25-backends) · [Guias →](/guides/)
