# Repository Governance

1. **Branches**:
   - `main`: the main branch. Current practice (solo developer): direct push
     with all gates green before the push; once there are collaborators,
     mandatory PRs with a protected branch take effect.
   - Working-branch naming pattern: `feat/*`, `fix/*`, `chore/*`, `docs/*`, `refactor/*`.

2. **Commits**:
   - Descriptive messages in the imperative mood, referencing the Goal when there is one
     (`P01-G02: ...`); Conventional Commits (`type(scope): ...`) recommended.

3. **Pull Requests & Merge** (once there are collaborators):
   - All code enters `main` via PR.
   - Merge strategy: `squash`, with the working branch deleted after the merge.
   - CI quality gates must pass 100%: `cargo fmt --check`, `cargo check`,
     `cargo clippy -D warnings`, `cargo test`, `cargo doc`, `prumo validate`,
     `prumo doctor` (see `docs/testing/ci-tiers.md` for the tiers).
