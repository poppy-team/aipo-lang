# Observabilidade do Toolchain

Não há serviço, endpoints ou traces distribuídos: a observabilidade do Aipo é
o próprio contrato de diagnósticos + artefatos de medição versionados.

1. **Diagnósticos estruturados**: todo erro sai em stderr como texto
   `arquivo:linha:coluna` ou `--message-format=jsonl` (schema estável em
   `docs/diagnostics/catalog.md`); programa nunca mistura stdout com erro.
2. **Códigos de saída contratuais**: `0` sucesso, `1` falha de linguagem,
   `2` erro de uso (ver `docs/reference/cli.md`).
3. **Baselines de performance**: `docs/performance/baseline.md` (medianas/MAD
   por workload) + JSON de baseline do `aipo-bench` para comparação.
4. **Cobertura**: relatório LLVM via `docs/testing/coverage.sh` (navegação, não meta).
5. **Evidências por goal**: `docs/evidence/` registra o que foi provado, com comandos.
