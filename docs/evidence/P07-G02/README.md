# P07-G02: estado e recuperação

**Data:** 2026-10-09 · **Base:** f0a0d70d176be031cd4b600fde1df6179429fb9a.
**Status:** DRAFT, publicação parcial de documentação.

## Bloqueio observado

O checkout estava em `feat/runtime-and-tooling-completion`, com alterações locais ainda sem commit/push. Execução e leitura de arquivos passaram a aguardar indefinidamente; depois apply_patch retornou explicitamente:

```text
exec-server connection attempt failed:
environment registry request failed
(409 Conflict, environment_offline): Environment is not connected.
```

Este commit preserva documentação pela API GitHub. Não preserva todos os arquivos fonte nem os CSV/artefatos locais. Não houve aprovação automática rejeitada: o bloqueio é a desconexão do ambiente. A autorização do usuário para commit/push/PR continua válida; sua instrução para não executar testes também.

## Checagens observadas

| Check | Resultado observado | Escopo |
|---|---|---|
| cargo clippy --workspace --all-targets --locked -- -D warnings, Rust 1.99 | exit 0 | Último snapshot, incluindo scheduler persistente e regressão de Task |
| cargo +1.96.0 check --workspace --all-targets --locked | exit 0 | Antes da última mudança de scheduler |
| cargo +1.96.0 check -p aipo-cli --all-targets --locked --no-default-features, com shell/web/embedded e sem alias | exit 0 | Antes da última mudança de scheduler |
| cargo +1.96.0 doc --workspace --no-deps --locked, RUSTDOCFLAGS=-D warnings | exit 0 após limpar cache de aipo-vm | Antes da última mudança de scheduler; erro antigo em fingerprint foi removido, não ignorado como prova |
| cargo fmt --all | exit 0 | Formatação aplicada no último snapshot; check final pendente |
| C header C11/C++17, -fsyntax-only | exit 0 | Sem link/executar programa; 29 exports identificados no source estavam declarados no header |
| Sintaxe Python de scripts release/perf | OK por ast.parse | Sem executar instalador/empacotador |
| Tests, conformance, Miri, fuzz, ASan | Não executados | Instrução explícita do usuário |
| Workflows e builds de outros SOs | Não observados | Não certificados |

Cargo check/all-targets e Clippy compilam alvos; não executam suites.

## Benchmark preliminar e limites

Foi criado exemplo comum às duas revisões com 20 mil calls Reg, caller de nove instruções e um registrador no callee. Casos Int e String usados; baseline f0a0d70d, Rust 1.99, CPU 0, três lotes × cinco pares, warmup removido. Medianas preliminares relatadas pelo driver: Int 17.064 ms baseline / 2.927 ms candidata; String 24.399 ms / 3.859 ms.

Esses números **não certificam o código final**: a sessão sofreu alterações depois da medição, o host também compilava durante parte da coleta e os CSV não puderam ser publicados. Repetir o benchmark final em ambiente sem compilação concorrente e anexar dados brutos/hashes antes de qualquer claim final de ganho.

## Retomada

1. Recuperar o ambiente/checkpoint; não recriar sobre main apagando alterações locais.
2. Conferir inventário no [registro JSON](recovery.json) e no [guia](../../development/runtime-and-tooling-guide.md).
3. Revisar RHS aliases/shadowing do `is` Wasm, budget counting e todos os paths de unwind.
4. Repetir checks de compilação final/MSRV/features/docs/header; não executar testes sem mudança da instrução do usuário.
5. Rebuild das duas revisões e benchmark sem build concorrente; anexar CSV/hashes.
6. Publicar código e documentação, conferir tree remota e atualizar esta PR. Goal permanece DRAFT até gates reais.

A recuperação preserva a autorização e as pendências, não transforma implementação local em implementação publicada.
