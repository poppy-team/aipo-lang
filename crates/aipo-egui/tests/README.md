# aipo-egui Tests Directory

Este diretório contém a suíte de testes de integração e conformidade do binding `egui`:

- `egui_integration_tests.rs`:
  - Ciclo de vida geracional com detecção de stale handles.
  - Verificação de segurança e revogação dinâmica de capacidades (`egui`).
  - Execução de scripts Aipo com widgets imediatos.
  - Extração de primitivas e verificação de hover/clique de ponteiro.
