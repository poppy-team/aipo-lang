# Governança de Repositório

1. **Branches**:
   - `main`: branch principal. Prática atual (desenvolvedor solo): push direto
     com todos os gates verdes antes do push; quando houver colaboradores,
     passa a valer PR obrigatório com branch protegida.
   - Padrão de branches de trabalho: `feat/*`, `fix/*`, `chore/*`, `docs/*`, `refactor/*`.

2. **Commits**:
   - Mensagens descritivas no imperativo, referenciando o Goal quando houver
     (`P01-G02: ...`); Conventional Commits (`tipo(escopo): ...`) recomendado.

3. **Pull Requests & Merge** (quando houver colaboradores):
   - Todo código entra em `main` via PR.
   - Estratégia de merge: `squash` com branch de trabalho deletada após o merge.
   - Quality gates do CI devem passar 100%: `cargo fmt --check`, `cargo check`,
     `cargo clippy -D warnings`, `cargo test`, `cargo doc`, `prumo validate`,
     `prumo doctor` (ver `docs/testing/ci-tiers.md` para os tiers).
