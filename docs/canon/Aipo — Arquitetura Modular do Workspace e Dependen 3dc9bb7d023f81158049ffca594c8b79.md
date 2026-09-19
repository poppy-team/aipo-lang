# Aipo — Arquitetura Modular do Workspace e Dependency Rules

<aside>
🧩

A modularização da Aipo é uma regra arquitetural: cada crate possui responsabilidade e direção de dependência explícitas. O objetivo é permitir evolução independente, testes locais, contexto pequeno para code agents e substituição de providers sem efeitos sistêmicos.

</aside>

## Topologia baseline

```
crates/
├── aipo-source
├── aipo-lexer
├── aipo-syntax
├── aipo-ast
├── aipo-hir
├── aipo-sema
├── aipo-ir
├── aipo-bytecode
├── aipo-vm
├── aipo-runtime
├── aipo-host
├── aipo-stdlib
├── aipo-diagnostics
├── aipo-formatter
├── aipo-lsp
├── aipo-cli
├── aipo-js
└── aipo-poppy
```

A lista é baseline, não licença para criar crates sem necessidade. Splits/merges exigem boundary rationale.

## Direção de dependência

```
source
  ↓
lexer → syntax → ast → hir → sema → ir
                              ↓      ├→ bytecode → vm → runtime
                              ↓      └→ js
                         diagnostics

host contracts ← runtime/vm
      ↑
 stdlib host adapters
      ↑
    poppy
```

Formatter/LSP/CLI consomem contracts; não se tornam fonte semântica.

## Regras

- nenhuma dependência circular;
- frontend crates não dependem de runtime/backend;
- `aipo-ir` é target-neutral;
- `aipo-bytecode` não conhece Poppy;
- `aipo-host` define abstrações gerais; `aipo-poppy` adapta;
- `aipo-stdlib` separa portable contract de providers host-specific;
- `aipo-diagnostics` mantém model/codes independente de terminal/LSP rendering;
- `aipo-lsp` não duplica analyzer; reutiliza compiler services;
- `aipo-cli` é thin orchestration layer;
- parser canônico não depende de Tree-sitter;
- generated bindings nunca viram segunda fonte manual de tipos.

## Crate contract obrigatório

Cada crate declara: mission, owns, does-not-own, public API, dependencies allowed, forbidden dependencies, stable data types, error/fault boundary, concurrency assumptions, unsafe allowance, performance budget, tests, update triggers e examples.

## Public API budget

`pub` só quando outro crate realmente precisa. Preferir `pub(crate)`/private. Reexports são deliberadas e documentam façade. Não criar prelude interno gigante que obscureça origem dos símbolos.

## Dependency inversion

Boundaries de host, filesystem, clock, scheduler e external services dependem de contracts estreitos definidos do lado consumidor. Implementações específicas ficam nas bordas. Não transformar tudo em trait: abstração nasce de substituição real ou boundary real.

## Data ownership

Cada representation possui owner claro. Syntax nodes pertencem a syntax; HIR ao HIR; semantic facts ao sema; IR ao IR; bytecode ao bytecode. Não reaproveitar type de camada anterior só para evitar mapping se isso acoplar fases semanticamente distintas.

## Cross-crate mapping

Mappings são explícitos e testados. Stable IDs/spans são preservados quando necessários a diagnostics/source maps. Backend não deve reler source text para reconstruir fatos perdidos pelo frontend.

## Generated artifacts

Schemas/grammar facades/bindings/docs podem ser gerados, mas generator input é canônico e versionado. CI regenera e detecta diff não commitado quando generated files forem versionados.

## Context boundary para agents

Uma task deve normalmente envolver poucos crates. Se um agent precisa modificar metade do workspace para uma feature pequena, considerar architecture smell e executar dependency/impact review.

## God-crate guardrail

Sinais de split: múltiplos domínios independentes, build/test hotspots, APIs públicas não relacionadas, contextos enormes recorrentes, cycles evitados por hacks. Sinais de merge: crate sem domínio próprio, thin pass-through sem boundary, abstração prematura ou fragmentação que piora entendimento.