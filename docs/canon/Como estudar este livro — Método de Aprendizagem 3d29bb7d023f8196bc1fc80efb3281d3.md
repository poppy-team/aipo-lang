# Como estudar este livro — Método de Aprendizagem

<aside>
🎓

**Princípio central:** aprender Odin construindo a Aipo e aprender compiladores, VMs e runtimes construindo a Aipo.

</aside>

## Método oficial

- Aprendizagem orientada por projeto e necessidade real.
- Progressão em espiral: conceitos retornam em contextos cada vez mais profundos.
- PRIMM: prever → executar → investigar → modificar → criar.
- Exemplos resolvidos antes de tarefas totalmente abertas.
- Scaffolding/fading: o suporte diminui conforme a competência aumenta.
- Retrieval practice e spaced practice integradas ao desenvolvimento.
- Um conceito estruturalmente difícil novo por etapa como orçamento cognitivo padrão.
- Recursos low-level de Odin entram somente quando resolvem um problema já compreendido da Aipo.

## Formato padrão de uma etapa

1. Recuperar um conceito anterior.
2. Apresentar um problema concreto da Aipo.
3. Introduzir o mínimo de teoria necessário.
4. Mostrar um exemplo funcionando.
5. Prever e executar.
6. Investigar o código.
7. Modificar uma parte pequena.
8. Criar uma pequena variação.
9. Testar.
10. Explicar o que foi construído.
11. Registrar no livro.

## Progressão de Odin

### Nível 1 — frontend

`package`, `import`, tipos básicos, procedures, `enum`, `struct`, arrays, slices, strings, fluxo de controle e o ciclo `odin check` / `odin run` / `odin test`.

### Nível 2 — compilador

Dynamic arrays, maps, organização em packages, allocators explícitos quando necessários, `context.allocator`/`context.temp_allocator`, arenas para dados com lifetime agrupado e APIs de diagnóstico/teste.

### Nível 3 — VM

Ponteiros, unions/representações variantes quando úteis, casts, alinhamento, operações de bits, layout de memória, stacks/frames e representação compacta de `Value`.

### Nível 4 — runtime e GC

Separação entre memória do host e **Aipo Heap**, allocators customizados, arenas, mark/sweep, raízes, tracing, write barriers quando o GC incremental exigir, sanitizers e testes de stress do collector.

### Nível 5 — embedding, games e Web

C ABI/FFI apenas para integrar o host e bibliotecas nativas, bindings do ecossistema Odin quando úteis, fibers/event loop, APIs de game host e emitter JavaScript. Isso não cria um backend C/native para programas Aipo.

### Nível 6 — somente quando justificado

Recursos avançados de Odin, otimizações de bytecode/VM, especialização, caches, profiling e eventuais técnicas de GC mais sofisticadas entram apenas quando um benchmark ou problema real justificar.

## Ciclo de feedback obrigatório

```
implementar pequena mudança
    ↓
odin check
    ↓
odin test
    ↓
golden/conformance tests
    ↓
GC/VM stress tests quando aplicável
    ↓
sanitizers quando aplicável
    ↓
benchmark somente depois de correto
```

O test runner de Odin deve ser aproveitado como parte central do aprendizado e da segurança da implementação, incluindo seu rastreamento de memória durante testes.

## Regra de documentação

O livro não será um diário bruto. O material deve ser reorganizado para permanecer ensinável: conceitos relacionados podem ser consolidados, capítulos grandes podem ser divididos e decisões superadas devem ser marcadas sem apagar a história quando ela tiver valor pedagógico.

## Regra de separação mental

Durante o estudo, manter sempre três perguntas independentes:

- **Aipo:** qual comportamento queremos oferecer ao usuário?
- **Compilador/VM:** como esse comportamento será representado e executado?
- **Odin:** qual é o menor recurso da linguagem hospedeira necessário para implementar essa etapa?

Isso impede que detalhes de gerenciamento manual de memória de Odin vazem para a superfície simples e gerenciada da Aipo.