---
title: "Estruturas"
description: "Representar dados por campos nomeados."
---
# Estruturas

**III · Construir** · Capítulo 13 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Representar dados por campos nomeados.

## Entenda o conceito

Uma `struct` agrupa valores relacionados. Campos sem anotações são usados pelo corpus atual; os contratos tipados de campo ainda precisam ser confrontados com a gramática-alvo.

## Experimente

```aipo
struct Pessoa {
    nome
    idade
}
let p = Pessoa{nome: "Lia", idade: 22}
io.println(p.nome)
```





## Exercício

Acrescente um campo `cidade`.

**Critério de conclusão:** A construção e a leitura usam os mesmos nomes.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/24_idiomatic_aipo_showcase.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Módulos e arquivos](/learn/12-modulos) · [Métodos e comportamento →](/learn/14-metodos)
