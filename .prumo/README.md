# Diretório Operacional do Prumo (`.prumo/`)

## O que é este diretório?
Contém metadados de execução, caches de desempenho, histórico analítico e estado derivado mantidos pelo Prumo CLI.

## Para que serve?
Isola todo estado temporário ou derivado do código-fonte canônico. Dados em `.prumo/runtime/` e `.prumo/cache/` são voláteis e não devem ser versionados no Git (com exceção do histórico estruturado em `history/`).

## Inventário
- `cache/`: Cache de resoluções de esquemas e pacotes para acelerar comandos do CLI.
- `history/`: Histórico cumulativo de inteligência do projeto, tokens consumidos e métricas.
- `runtime/`: Checkpoints de execução, sessões ativas e cápsulas de contexto.
