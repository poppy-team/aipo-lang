---
title: "Pacotes e lockfiles"
description: "Organizar dependências de forma reproduzível."
---
# Pacotes e lockfiles

**IV · Avançado** · Capítulo 24 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Organizar dependências de forma reproduzível.

## Entenda o conceito

Um módulo corresponde a um arquivo; um pacote tem manifesto `aipo.toml`. O lockfile e o cache registram resolução e integridade. Dependências remotas não devem ser buscadas sem intenção explícita.

## Experimente

```toml
[package]
name = "exemplo"
version = "0.1.0"
```




O manifesto é escrito em TOML e deve ser salvo como `aipo.toml`.


## Exercício

Crie o manifesto e consulte `aipo package audit`.

**Critério de conclusão:** O bloco acima é TOML, não código Aipo.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/docs/manual/packages-and-modules.md)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Tarefas e async](/learn/23-concorrencia) · [VM, JavaScript e Wasm →](/learn/25-backends)
