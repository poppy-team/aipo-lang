# aipo-egui Source Directory

Este diretório contém a implementação dos componentes internos do crate `aipo-egui`:

## Módulos
- `lib.rs`: Ponto de entrada do crate, re-exportações públicas e declaração `#![forbid(unsafe_code)]`.
- `context.rs`: Wrapper em torno de `egui::Context`, gerenciamento de pilha de `Ui` e ciclo de vida de frames.
- `shapes.rs`: Conversor universal de primitivas visuais do `egui::epaint` em estruturas de dados agnósticas Aipo (`Dict` / `List`).
- `schema.rs`: Definição de capacidades e validação do perfil de host `egui`.
- `adapter.rs`: Funções nativas da VM Aipo, tabela global de sessões com `HandleTable` e dicionário do módulo.
