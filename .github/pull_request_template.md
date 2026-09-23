## Summary

Describe the change and the problem it solves.

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Relevant documentation and conformance fixtures were updated

## Security and supply chain

- [ ] No secrets, credentials, private keys, tokens, or production data are included
- [ ] New/changed dependencies are necessary and their license/source is acceptable
- [ ] Untrusted input is validated at the appropriate boundary
- [ ] The change preserves the workspace-wide `unsafe_code = "forbid"` policy
- [ ] Security-sensitive behavior has regression coverage
