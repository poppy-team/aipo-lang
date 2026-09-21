# Padrões de Codificação e Engenharia (Rust)

Subordinado ao canon `Aipo — Rust Engineering Standard …` e aos lints do
workspace (`Cargo.toml`: `unsafe_code = "forbid"`, clippy `all/correctness/
suspicious/complexity/perf/style` como `deny`).

1. **Formatação e Estilo**:
   - Todo código Rust passa por `rustfmt` (`cargo fmt --all --check` no gate).
   - Avisos do compilador e do clippy são erro (`-D warnings` no gate).
2. **Tipagem e Erros**:
   - Tipagem estrita em todas as assinaturas públicas (`Option`/`Result`
     explícitos, sem `unwrap` em caminhos de usuário).
   - Erros de usuário Aipo são `Result`/`Diagnostic`/`VmError` — nenhum `panic`
     escapa como erro de usuário (garantido por fuzz + `catch_unwind` nos
     harnesses hostis).
   - `unsafe` proibido no workspace; dependências com `unsafe` exigem linha de
     exceção aprovada em `docs/security/security-contract.md`.
3. **Documentação no Código**:
   - Comentários explicam o *porquê*, nunca o *o quê*.
   - Itens públicos têm rustdoc (`missing_docs = warn`); crates têm `description`.
4. **Testes**:
   - Toda mudança de comportamento chega com fixture/regressão (corpus,
     goldens ou teste unitário); ver `docs/development/testing-strategy.md`.
   - Testes determinísticos (sementes fixas); nada dependente de wall-clock
     como gate; saída de programa capturada sob lock serializado
     (ver `docs/testing/concurrency-audit.md`).
