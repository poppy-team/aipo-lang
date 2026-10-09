# Runtimes de referência: features, limites e próximos experimentos

**Rodada:** 2026-10-09. [Nove referências pinadas](../../studies/refs.json), leitura dirigida de source, sem builds/benchmarks upstream. Valores de tamanho ou velocidade anunciados por projetos não são medições desta revisão.

## Luau

**O que é:** linguagem e runtime com compiler, análise estática e VM em componentes separados.

**Como funciona:** [`VM/src/lobject.h`](https://github.com/luau-lang/luau/blob/74f768309c380d40601b28c4c4cdda27da415cef/VM/src/lobject.h#L46) define `TValue` com payload, campos extras e tag. [`lbuffer.h`](https://github.com/luau-lang/luau/blob/74f768309c380d40601b28c4c4cdda27da415cef/VM/src/lbuffer.h#L6) define limite de buffer e cálculo de armazenamento próprio. Isso difere de compartilhar o mesmo layout com strings.

**Pergunta fechada:** buffer é categoria concreta de runtime, com limites explícitos; não é apenas texto com outro nome.

**Lição → ação:** Aipo preserva `Bytes` distinto de `String`; RegVM agora exige índice `Int` e valor `Byte` antes da escrita. Estudo aprofundado de `Analysis`/`Compiler`, diagnósticos e GC incremental continua pendente, sem claim de paridade com a Aipo.

## Janet

**O que é:** runtime C com VM, fibers e GC, com representação de valores configurável.

**Como funciona:** [`janet.h`](https://github.com/janet-lang/janet/blob/246951c8090aa714342be246a7abdb7312cf7dae/src/include/janet.h#L311) contém gates de nanboxing 32/64 bits, condições de arquitetura e premissas de endereço/alinhamento. [`vm.c`](https://github.com/janet-lang/janet/blob/246951c8090aa714342be246a7abdb7312cf7dae/src/core/vm.c#L1321) expõe step/continue para fibers; [`janet_collect`](https://github.com/janet-lang/janet/blob/246951c8090aa714342be246a7abdb7312cf7dae/src/core/gc.c#L578) é uma entrada explícita para investigar coleta e raízes.

**Pergunta fechada:** representação compacta depende do alvo e das premissas de ponteiro. Fiber não equivale automaticamente ao scheduler determinístico de `Task` da Aipo.

**Lição → ação:** manter NaN boxing como experimento separado, com gates e justificativa de portabilidade; preservar o Modelo B de falhas ao estudar suspensão. Não houve implementação de fibers ou collector Janet nesta revisão.

## wasm3

**O que é:** interpretador WebAssembly com configuração C e metacódigo próprio.

**Como funciona:** [`m3_config.h`](https://github.com/wasm3/wasm3/blob/28ecb9af6d2040e474a70f7cb7f43666740141fb/source/m3_config.h#L14) define limites de stack, páginas de memória, slots e validação. Configurações de tail calls dependem de manter ou eliminar frame nativo; as condições não permitem concluir portabilidade irrestrita.

**Pergunta fechada:** o custo de um runtime não se resume ao tamanho de seu executável; limites e flags de compilação fazem parte do perfil avaliado.

**Lição → ação:** qualquer experimento em Aipo precisa registrar build flags, memória, validação, imports e checksum. Não foi entregue integração wasm3 nem certificado um perfil de poucos kilobytes.

## WAMR

**O que é:** runtime WebAssembly com interpretadores, AOT/JIT e integrações de host configuráveis.

**Como funciona:** [`runtime_lib.cmake`](https://github.com/wasm-micro-runtime/wasm-micro-runtime/blob/38b044a676893e678ceb19babaa431ca5e460114/build-scripts/runtime_lib.cmake) seleciona componentes; [`config_common.cmake`](https://github.com/wasm-micro-runtime/wasm-micro-runtime/blob/38b044a676893e678ceb19babaa431ca5e460114/build-scripts/config_common.cmake#L301) condiciona fast interpreter ao interpretador habilitado. Libc WASI e uvwasi são configurações separadas, não uma obrigação de todo build.

| Dimensão | Gate observado | Implicação para um experimento |
| --- | --- | --- |
| Interpretador | `WAMR_BUILD_INTERP` | Definir modo de execução |
| Fast interpreter | `WAMR_BUILD_FAST_INTERP` | Comparar a mesma carga com flags registradas |
| AOT/JIT | `WAMR_BUILD_AOT`, `WAMR_BUILD_JIT` | Separar startup, compilação e execução |
| WASI | `WAMR_BUILD_LIBC_WASI`, `WAMR_BUILD_LIBC_UVWASI` | Definir imports e permissões do host |
| Alvo | `WAMR_BUILD_TARGET` | Não transportar tamanho de RISC-V para x86-64 |

**Pergunta fechada:** há configurações com e sem WASI. WAMR é um candidato de experimento, não um “líder” medido nem a única opção pequena com WASI.

**Lição → ação:** produzir um spike isolado antes de propor troca de engine; registrar imports, footprint e licença por arquivo. Há arquivos com `Apache-2.0 WITH LLVM-exception`; a licença não deve ser reduzida a uma etiqueta única sem conferir o trecho reutilizado. Nenhum source upstream foi copiado para Aipo.

## wasmi

**O que é:** interpretador WebAssembly em Rust, com núcleo e adaptador WASI separados no workspace.

**Como funciona:** [`crates/wasmi/src/lib.rs`](https://github.com/wasmi-labs/wasmi/blob/2970aa871cc1001b57b267ccecdcd1e42306199e/crates/wasmi/src/lib.rs#L87) declara `#![no_std]`, usa `alloc` e habilita `std` por feature. O [manifest do núcleo](https://github.com/wasmi-labs/wasmi/blob/2970aa871cc1001b57b267ccecdcd1e42306199e/crates/wasmi/Cargo.toml#L37) inclui gates como `std`, `wat`, `validate`, `memory64`, `auto-dispatch`, `portable-dispatch`, `indirect-dispatch` e `deterministic`.

O [adaptador WASI](https://github.com/wasmi-labs/wasmi/blob/2970aa871cc1001b57b267ccecdcd1e42306199e/crates/wasi/Cargo.toml) depende do núcleo com `std`, de `wasi-common` e de `wiggle`. A [licença do workspace](https://github.com/wasmi-labs/wasmi/blob/2970aa871cc1001b57b267ccecdcd1e42306199e/Cargo.toml#L25) é `MIT/Apache-2.0`, não exclusivamente Apache-2.0.

**Perguntas fechadas:** a frase “wasmi com WASI” não invalida `no_std` do núcleo; integração WASI e núcleo são componentes distintos. Disponibilidade de uma feature no source não certifica um build em todos os alvos.

**Lição → ação:** incluir wasmi no comparativo com WAMR antes de escolher runtime para shell/nano. Esta revisão aplica o conceito de limites opcionais ao runner Wasmtime existente, sem trocar a engine. Builds `no_std`, consumo de memória, WASI e desempenho comparável permanecem sem medição.

## Protocolo de comparação futuro

Fixar o mesmo `.wasm`, entrada, checksum e imports; medir cold/warm start separadamente; registrar flags, dependências, alvo, toolchain e tamanho; verificar licença e erro por limite. Não desabilitar validação em um candidato e mantê-la em outro sem registrar a diferença.

Aipo continua com Wasmtime opcional. A separação de compiler e runner, fuel e erro explícito de feature desabilitada foram entregues; seleção de uma engine alternativa é proposta, não decisão concluída.
