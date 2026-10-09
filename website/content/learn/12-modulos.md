---
title: "Módulos e arquivos"
description: "Compartilhar funções entre arquivos."
---
# Módulos e arquivos

**II · Fundamentos** · Capítulo 12 de 26 · [Índice do livro](/learn/)

> **Verificação:** trecho didático ainda não executado nesta migração. A [referência de sintaxe](/reference/syntax) diferencia o alvo da implementação atual.

## Objetivo

Compartilhar funções entre arquivos.

## Entenda o conceito

Cada arquivo `.aipo` é um módulo. `export` declara o que está disponível para outros módulos; a resolução de nomes importados possui regras específicas e não deve ser presumida igual à de Python.

## Experimente

```aipo
# utils.aipo
fn dobro(n) { return n * 2 }
export dobro

# main.aipo (arquivo separado)
import utils
io.println(dobro(5))
```

Os dois nomes de arquivos nos comentários indicam **arquivos separados**. Não execute este bloco inteiro como um único arquivo.





## Exercício

Separe os dois arquivos e experimente importar uma função não exportada.

**Critério de conclusão:** Os comentários indicam arquivos distintos, não um script único.

## Aprofundamento

- [Código existente ou contrato relacionado](https://github.com/poppy-team/aipo-lang/blob/main/examples/05_modules/main.aipo)
- [Referência de sintaxe](/reference/syntax)
- [Como ler mensagens de erro](/guides/troubleshooting)

---

[← Escopo e mutabilidade](/learn/11-escopo) · [Estruturas →](/learn/13-estruturas)
