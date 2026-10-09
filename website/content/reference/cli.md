---
title: Referência prática do CLI
---
# Referência do CLI

Use `cargo run -q -p aipo-cli -- <comando>` na raiz do repositório ou o binário `aipo` compilado.

| Tarefa | Comando | Observações |
| --- | --- | --- |
| Executar | `aipo run programa.aipo` | Usa a VM de referência por padrão |
| Validar | `aipo check programa.aipo` | Analisa sem executar |
| Testar | `aipo test` | Descobre e executa testes Aipo |
| Formatar | `aipo fmt arquivo.aipo` | Pode modificar arquivo |
| Conferir formato | `aipo fmt --check arquivo.aipo` | Para CI |
| Construir JS | `aipo build programa.aipo --target js --out dist/` | Dependente da cobertura JS |
| Construir Wasm | `aipo build programa.aipo --target wasm --out dist/` | Dependente das features Wasm |
| Inspecionar | `aipo disasm programa.aipo` | Mostra representação emitida |
| Fechar lockfile | `aipo package lock .` | Opera no pacote |
| Auditar lockfile | `aipo package audit .` | Sem modificar resolução |
| Verificar cache | `aipo package cache verify DIR` | Verifica integridade |
| Limpar cache | `aipo package cache prune DIR --lock aipo.lock --apply` | Mutação explícita |

## Motores de execução

`aipo run arquivo.aipo --engine=reg` escolhe a Register VM experimental. Não assuma equivalência com a VM padrão. `--wasm` seleciona o percurso Wasm quando presente na build.

## Esquema de host

`aipo check arquivo.aipo --ahs esquema.json` carrega descrições para a invocação; isso não dá acesso efetivo a capacidades.

## Códigos de saída

- `0`: êxito.
- `1`: falha de linguagem, teste ou runtime.
- `2`: erro de uso do comando.

A [especificação histórica detalhada da CLI](https://github.com/poppy-team/aipo-lang/blob/main/docs/reference/cli.md) tem flags adicionais e restrições. **O `--help` do binário compilado para seu commit é a autoridade do que está disponível.**
