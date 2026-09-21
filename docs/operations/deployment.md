# Guia de Implantação (Deployment)

Aipo é um toolchain CLI (binário `aipo`), não um serviço: não há staging,
migração de banco ou rollout — há gates, versão e checksum.

## Requisitos de Release

1. Gates `fast` 100% verdes (`docs/testing/ci-tiers.md`): fmt, check, clippy
   `-D warnings`, suite completa, doc, `prumo validate/doctor`.
2. `cargo audit` e `cargo deny check` limpos.
3. Versão atualizada (`Cargo.toml` workspace + crates afetados) e notas no `CHANGELOG.md`.

## Procedimento

- `cargo build --release -p aipo-cli`; publicar checksum (SHA-256) do binário.
- Smoke test pós-build: `aipo run examples/01_fizzbuzz.aipo` e um `aipo build` +
  `node` de conferência devem reproduzir as saídas commitadas.
- Rollback = republicar o binário da tag anterior (tags Git por release).
