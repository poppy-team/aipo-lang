# VMs de referência: leitura pinada e aplicação à Aipo

**Rodada:** 2026-10-09. Fontes: [manifest](../../studies/refs.json). Os nove checkouts foram obtidos em commits completos, com HEAD e origin conferidos. Esta nota registra leitura dirigida, sem compilar ou medir os upstreams.

## Wren

**O que é:** linguagem com VM compacta e embedding C. A arquitetura de fibers, frames e upvalues fornece uma referência concreta para lifetimes de execução.

**Como funciona:** em [`wren_vm.c`, `captureUpvalue`/`closeUpvalues`](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_vm.c#L244), upvalues abertos ficam em uma lista ordenada pelo slot. Fechar um upvalue copia o valor para armazenamento próprio e muda seu apontamento. O retorno fecha upvalues do frame antes de descartar slots.

[`CallFrame`](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_value.h#L284) guarda IP, closure e base de stack. A VM atualiza caches locais de frame quando muda a ativação. [`WREN_COMPUTED_GOTO`](https://github.com/wren-lang/wren/blob/99d2f0b8fc2686134b32b18166e037639f7e9f2c/src/vm/wren_vm.c#L890) escolhe dispatch por labels no compilador compatível; existe fallback com `switch`.

**Perguntas fechadas:** captura exige uma relação explícita entre slot e lifetime; retorno precisa fechar capturas antes de invalidar slots; computed goto é uma escolha condicionada de build, não uma característica que se copia literalmente para Rust seguro.

**Lição → ação:** nesta revisão, RegVM isola handlers e restaura estado do caller. Próxima etapa: modelar frames com base/slots e células de captura, mantendo a semântica do canon. Leitura integral de compiler, GC e API C, e comparação de tempos, continuam pendentes. Nenhum código C foi incorporado.

## QuickJS-ng

**O que é:** engine JavaScript embutível. Serve como referência para organização de valores, nomes de propriedades, ownership e coleta de ciclos, não como proposta de adotar a semântica de JavaScript.

**Como funciona:** o [`quickjs.h`](https://github.com/quickjs-ng/quickjs/blob/dad13e33cee39b910de2e45ea940c08baf42dbf9/quickjs.h#L208) distingue ownership de `JSValue`/`JSValueConst`; há representações de valor condicionadas por macros, incluindo union+tag. Não existe um único tamanho universal independente da configuração.

[`JSShapeProperty`/`JSShape`](https://github.com/quickjs-ng/quickjs/blob/dad13e33cee39b910de2e45ea940c08baf42dbf9/quickjs.c#L1102) associam atoms, flags e uma tabela de propriedades. [`JS_RunGC`](https://github.com/quickjs-ng/quickjs/blob/dad13e33cee39b910de2e45ea940c08baf42dbf9/quickjs.c#L7484) é o ponto para estudar o coletor; sua política inteira não foi auditada nesta reconstrução.

**Perguntas fechadas:** nomes internados e layouts compartilhados são mecanismos diferentes de valores da propriedade; representação compacta exige configuração e invariantes. Shapes não equivalem apenas a um `HashMap` adicionado a cada instância.

**Lição → ação:** manter a semântica de `Value` centralizada e medir custo de field lookup antes de introduzir shapes/slots na Aipo. Estudar ciclos e fronteiras C separadamente; `Rc` não resolve ciclos automaticamente. A política atual de ABI C já estava na base e não foi substituída.

## Lua

**O que é:** runtime embutível com valores tagged, stack de slots e closures com upvalues.

**Como funciona:** [`lobject.h`](https://github.com/lua/lua/blob/0b29f408433e92953cc72b1d3e06c7ac8139e439/lobject.h#L49) define payload e tag de `TValue`. [`luaF_findupval`](https://github.com/lua/lua/blob/0b29f408433e92953cc72b1d3e06c7ac8139e439/lfunc.c#L85) percorre upvalues abertos por nível, reusa o existente ou cria outro. Uma lista ordenada ainda pode ter busca linear no número de capturas abertas percorridas.

[`luaV_execute`](https://github.com/lua/lua/blob/0b29f408433e92953cc72b1d3e06c7ac8139e439/lvm.c#L1210) recebe o mecanismo de dispatch condicionado por `LUA_USE_JUMPTABLE`. O source estudado é o commit do manifest, não uma release implicitamente fixada como “Lua 5.4”.

**Perguntas fechadas:** layout e custo de captura dependem da representação; a busca ordenada não vira O(1); o mecanismo de dispatch precisa ser analisado no alvo e build reais.

**Lição → ação:** estudar slots e capturas antes de remover a troca de 256 registradores de RegVM. Nesta rodada, os operandos e a restauração do caller foram corrigidos; a migração para janelas permanece proposta. Tamanho de `TValue` não foi medido em compilação C nesta rodada.

## LuaJIT

**O que é:** implementação com interpretador, JIT e representação compacta de valores. É referência de encoding e frames, não dependência nova da Aipo.

**Como funciona:** [`lj_obj.h`](https://github.com/LuaJIT/LuaJIT/blob/c6ffc141a8762b41703f9287d63d93622a13dd8f/src/lj_obj.h#L151) define `BCIns` como `uint32_t`, além de `TValue` e representações condicionadas por `LJ_GC64`/`LJ_FR2`. [`lj_bc.h`](https://github.com/LuaJIT/LuaJIT/blob/c6ffc141a8762b41703f9287d63d93622a13dd8f/src/lj_bc.h#L25) fixa faixas e papéis ABC/AD/AJ. [`func_finduv`](https://github.com/LuaJIT/LuaJIT/blob/c6ffc141a8762b41703f9287d63d93622a13dd8f/src/lj_func.c#L37) reusa células em lista ordenada de upvalues.

**Perguntas fechadas:** largura da palavra não define por si a faixa de todos os operandos; formatos e papéis devem ser descritos por opcode. `GCRef` não deve ser anunciado como sempre 32 bits, pois existem configurações distintas.

**Lição → ação:** documentar papéis de B, checar constantes e offsets e usar opcodes de iteração com C completo. NaN boxing e dispatch/JIT seguem experimentos futuros; não há medida nesta revisão que justifique substituir o modelo seguro de `Value`.

## Limites comuns e continuidade

A inspeção cobre as regiões citadas. Uma análise integral de GC, compilação, FFI, reentrância e portabilidade exige outra rodada. Referências servem para formular hipóteses e invariantes, não para anunciar ganho de performance, memória ou maturidade da Aipo.

Consulte [lições para Aipo](lessons-for-aipo.md) e o [guia de reimplementação](runtime-hardening-guide.md) para o mapeamento entre source estudado e mudança efetivamente entregue.
