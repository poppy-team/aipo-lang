# Estudos de implementação Aipo

Esta rodada relaciona inspeção de fontes pinadas às correções e às próximas propostas. Leituras de código não são resultados de benchmarks.

- [Guia master para LLMs](llm-study-guide.md): metodologia, limites, corpus e fases.
- [Guia de uso e reimplementação](runtime-hardening-guide.md): mudanças concretas desta revisão.
- [VMs de referência](reference-vms.md): Wren, QuickJS-ng, Lua e LuaJIT.
- [Runtimes de referência](reference-runtimes.md): Luau, Janet, wasm3, WAMR e wasmi.
- [Lições para Aipo](lessons-for-aipo.md): ações implementadas e propostas pendentes.
- [Manifest de fontes](../../studies/refs.json): SHAs completos das nove referências.

Use `./studies/fetch-refs.sh` na raiz para obter o corpus fora do repositório e `--verify` para conferir um checkout já materializado sem rede. Não copie upstreams para o workspace Aipo nem transforme tamanhos anunciados por terceiros em medidas da nossa implementação.
