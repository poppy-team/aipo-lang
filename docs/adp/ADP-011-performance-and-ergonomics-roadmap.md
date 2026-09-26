# ADP-011 — Roteiro de Performance Estrutural e Ergonomia de Linguagem

**Status:** accepted
**Date:** 2026-09-26
**Related:** `docs/performance/baseline.md`, `docs/performance/cross-language.md`, `docs/adp/ADP-008-v0.1.0-language-release.md`, `crates/aipo-vm/`, `crates/aipo-bytecode/`
**Authority:** subordinate ao canon; define a estratégia arquitetural de médio e longo prazo para superação dos gargalos de performance e evolução ergonômica da linguagem Aipo.

## A. Contexto e Motivação

Durante a fase P04 e início de P05, sete experimentos locais de micro-otimização (pool de nomes, layout denso com 2 vetores, cache de flags fixed por slot, cache de tipos guarded, cache de metadata de métodos) falharam em produzir ganhos reprodutíveis e foram revertidos.

A auditoria arquitetural aprofundada comprovou que o interpretador da Aipo opera com um **piso estrutural de custo por opcode de ~100ns (~300 ciclos de CPU)**. Micro-otimizações locais são incapazes de transpor esse piso. Medições comparativas com `Wren 0.4.0` e `Luau 0.739` revelaram gaps entre 7x e 22x em computação aritmética e entre 10x e 14x em acesso a campos e despacho de métodos.

Adicionalmente, no aspecto ergonômico, a linguagem carece de falhas tipadas estruturadas (atualmente restritas a mensagens de texto plano), pattern matching com desestruturação e refinamento de tipo (type narrowing) estático.

## B. Decisões Arquiteturais de Performance

### P1. Compactação de `Value` para 16/24 Bytes
- **Decisão:** Remover estruturas com alocação inline de variantes da enum `Value`. A variante `Value::Native` passa a usar `Rc<NativeData>` ou índice de catálogo, e `Value::Function` utiliza inteiros compactos (`u32` para `entry_ip`, `u16` para `arity`).
- **Impacto:** O tamanho de `Value` cai de 48 bytes para 16 a 24 bytes, reduzindo a pressão de banda de memória em até 66% e quadruplicando a densidade de valores por cache line L1.

### P2. Fusão de Invocação de Método (`InvokeMethod`)
- **Decisão:** Compilar chamadas de método diretas `expr.method(args...)` como um opcode fundido `InvokeMethod`, em vez da sequência `GetField` + `Call`.
- **Impacto:** Eliminação total das 200.000 alocações efêmeras de `Rc<BoundMethodData>` observadas no workload `fields`.

### P3. Laço de Despacho com Variáveis Hoisted
- **Decisão:** Integrar o laço de execução dentro de uma função contínua em vez de invocar `self.step()` por instrução, promovendo ponteiros críticos (`ip`, `stack_base`, `code`) a variáveis locais registradas pela CPU.

### P4. Superinstruções e Especialização de Opcodes
- **Decisão:** Introduzir variantes especializadas `Call0..Call4` e `GetLocal0..GetLocal3` sem bytes extras de operando, aumentando a densidade do bytecode e reduzindo ciclos de decodificação.

## C. Decisões de Ergonomia de Linguagem

### E1. Falhas Estruturadas (Model B+)
- **Decisão:** Permitir que `fail` aceite qualquer valor ou instância de struct como payload da falha, preservando o rollback automático de invariantes e estendendo `attempt ... failed err` para inspeção tipada (`if err is MyError`).

### E2. Pattern Matching e Destructuring
- **Decisão:** Introduzir a expressão `match expr` com desestruturação estrutural de structs, listas/tuplas e tipos primitivos.

### E3. Type Narrowing Estático em `aipo-sema`
- **Decisão:** Implementar refinamento sensível ao fluxo em verificações `is`, elidindo a emissão de instruções `AssertContract` em runtime quando o tipo for garantido em tempo de compilação.

## D. Protocolo de Validação e Conformance

Todas as implementações decorrentes desta ADP devem:
1. Ser validadas com o harness pareado `scripts/perf/paired.sh` sob metodologia de tempo de CPU (`cpu_ab.py`) com múltiplos lotes.
2. Manter 100% de paridade diferencial entre a VM e o backend JavaScript (`aipo-js`).
3. Preservar a suíte completa de conformance e os 5 quality gates do projeto.
