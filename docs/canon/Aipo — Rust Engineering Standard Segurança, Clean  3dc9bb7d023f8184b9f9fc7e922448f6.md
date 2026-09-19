# Aipo — Rust Engineering Standard: Segurança, Clean Code e Qualidade

<aside>
🦀

**Política obrigatória:** a implementação oficial da Aipo segue Safe Rust por padrão, Clean Code pragmático, APIs pequenas, módulos coesos, testes por contrato e otimização orientada por profiling. Segurança, clareza e verificabilidade prevalecem sobre cleverness.

</aside>

## Toolchain e workspace

- Rust edition 2024.
- Toolchain pinada no repositório; upgrades são mudanças explícitas e testadas.
- Cargo workspace único para crates canônicas, compartilhando lockfile, lints e metadata quando possível.
- `workspace.dependencies` centraliza versões aprovadas quando isso reduz drift.
- `workspace.lints` define baseline comum.
- `cargo fmt`/rustfmt é autoridade de formatação; nenhuma formatação manual divergente.

Baseline de CI:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo nextest run --workspace
cargo doc --workspace --no-deps
```

Checks condicionais por risco incluem Miri, fuzzing, mutation testing, benchmarks, RustSec/cargo-audit e cargo-deny.

## Safe Rust first

Safe Rust é o baseline. `unsafe` só entra quando uma necessidade mensurável não puder ser atendida com Safe Rust ou quando uma FFI/library boundary inevitavelmente exigir.

Toda ocorrência de `unsafe` deve:

- viver no menor módulo possível;
- ter `// SAFETY:` explicando invariants e por que estão satisfeitos;
- possuir public safe wrapper sempre que possível;
- documentar preconditions/postconditions;
- ter testes negativos/edge cases;
- rodar sob Miri quando tecnicamente aplicável;
- possuir owner/contract explícito;
- ser incluída em security review.

Proibir `unsafe` por lint no workspace/crates onde não seja necessário; liberar somente em módulos aprovados. `unsafe_op_in_unsafe_fn` deve permanecer explícito.

## Clean Code — política permanente

Clean Code é aplicado pragmaticamente, sem dogma de “funções minúsculas a qualquer custo”. O objetivo é tornar intenção, domínio e invariants evidentes.

### Naming

- nomes expressam domínio e intenção;
- evitar abreviações não universais;
- `parse_module`, não `pm`;
- `resolved_symbol`, não `rs`;
- nomes booleanos formam perguntas (`is_valid`, `has_capability`, `can_escape_scope`);
- types são substantivos; funções são ações/transformações;
- evitar `Manager`, `Helper`, `Util`, `Common`, `Misc`, `Data`, `Thing` como substitutos de domínio.

### Functions

- uma responsabilidade coesa;
- inputs e efeitos explícitos;
- evitar parâmetros booleanos que mudem radicalmente comportamento; preferir enum/options semanticamente nomeados;
- early return para reduzir nesting quando melhora leitura;
- não extrair função apenas para reduzir contagem de linhas; extrair quando há conceito ou boundary real;
- side effects nas bordas; lógica semântica preferencialmente pura/testável.

### Types

- usar newtypes quando evitam mistura de IDs/units/domains;
- enums para estados fechados;
- typestate apenas quando reduz estados inválidos sem explodir complexidade;
- invariants importantes pertencem ao tipo/constructor;
- `pub` é opt-in: menor visibility possível.

## Error handling

- `Result` para falhas recuperáveis;
- `Option` para ausência legítima;
- panic somente para bug/invariant interno realmente impossível de continuar, nunca para input do usuário/source inválido/package inválido/host error esperado;
- evitar `.unwrap()`/`.expect()` em production paths; permitir em testes e invariants locais comprovados;
- preservar source errors ao adicionar contexto;
- errors de domínio têm tipos/códigos estáveis quando atravessam boundaries;
- parser/analyzer/runtime nunca devem vazar panic Rust como diagnostic Aipo.

## Ownership e borrowing

Não usar `clone()` como solução automática para o borrow checker. Cada clone de estrutura potencialmente grande deve ser intencional e justificável. Preferir ownership claro, borrowing curto e data flow simples.

Evitar `Rc<RefCell<T>>`/`Arc<Mutex<T>>` como arquitetura default. Interior mutability e shared synchronization só entram quando ownership model realmente exige; invariants e lock ordering devem ser documentados.

## Concurrency

- concorrência só quando há benefício comprovado;
- minimizar shared mutable state;
- preferir message passing, immutable snapshots ou ownership transfer quando adequado;
- locks curtos e ordem documentada;
- nenhum lock mantido sobre await/I/O salvo design explícito e auditado;
- deadlock/livelock/resource starvation entram na risk analysis.

## Dependencies

Toda dependência externa precisa responder: problema concreto, motivo para não usar std/internal, maturidade/manutenção, security history, licenças, custo transitivo, impacto de targets e custo de remoção.

Evitar dependências duplicadas que resolvem o mesmo domínio. Feature flags devem ser pequenas, documentadas e aditivas.

`cargo-audit`/RustSec verifica vulnerabilities conhecidas; `cargo-deny` governa advisories, licenses, bans/duplicates e sources. Exceções possuem justificativa, owner e data de revisão.

## API design

- APIs públicas pequenas e difíceis de usar incorretamente;
- constructors validam invariants;
- preferir semantic types a tuplas ambíguas;
- não expor representation interna quando contrato abstrato basta;
- evitar public generic abstraction antes de casos reais;
- traits pequenas, orientadas ao consumidor;
- trait não nasce apenas “para testabilidade” se função/struct simples resolve;
- mudanças públicas exigem compatibility/migration analysis.

## Modularidade

Crates e modules representam boundaries de domínio e ownership de responsabilidade. Modularização total não significa um arquivo por função; significa ausência de god modules, circular knowledge e responsabilidades misturadas.

Regras: dependency graph acíclico; frontend não depende de VM details; Core IR não depende de backend; diagnostics model não depende de renderer; host ABI não depende de Poppy; Poppy adapter depende de host contracts; stdlib portable não depende de target-specific implementation; generated code é isolado/reproduzível.

## Performance

Primeiro medir, depois otimizar. Evitar micro-otimização que degrade clareza sem benchmark. Hot paths documentados podem ter regras específicas. `Cow`, arenas, interning, small-vector strategies, unsafe optimization ou custom allocation só após workload demonstrar benefício.

## Testing

Unit para domain logic; snapshot/golden para tokens/syntax/diagnostics/HIR/IR/bytecode/formatter; property tests para invariants; differential VM↔JS; integration para pipeline/CLI/LSP/host; fuzz para parsers/decoders/boundaries; Miri para unsafe/data structures sensíveis; mutation testing onde coverage pode mascarar assertions fracas.

## Security coding

Validar lengths/indices antes de allocation/copy; limites explícitos para recursion/nesting/decoded sizes; no shell invocation por concatenação; política explícita para path traversal/symlinks; secrets não entram em Debug/log; fuzz failures viram regression tests.

## Comments e rustdoc

Comments explicam why/invariant/trade-off. Public APIs têm rustdoc e seções `# Safety`, `# Errors`, `# Panics` quando relevantes.

## Anti-overengineering

Antes de adicionar crate, trait, generic layer, macro, cache, thread pool, unsafe optimization ou codegen step: qual problema mensurável existe, por que a solução atual é insuficiente, qual o menor design reversível e como será removido se falhar?

## Definition of Done Rust

Fmt/check/clippy/tests verdes; nenhum warning; docs/contracts atualizados; dependência nova revisada; unsafe novo auditado; snapshots/conformance atualizados; checks de segurança/performance aplicáveis executados; public API documentada; debt residual registrada.