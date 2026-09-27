# Suíte de Testes do aipo.freya (`tests/`)

Este diretório contém os testes de unidade automatizados do framework `aipo.freya`.

---

## Arquivos de Teste

### `freya_test.aipo`
Suíte de testes de unidade cobrindo:
1. **Paleta de Cores e Parsing Hexadecimal:** Verificação de normalização RGB/RGBA para float de GPU e conversão de valores hexadecimais.
2. **Hooks Reativos (`use_state` / `set_state`):** Integridade de leitura, escrita e mutação encadeada de sinais de estado.
3. **Criação de Elementos e Componentes:** Validação da estrutura de dados da árvore, preservação de propriedades e vinculação de callbacks de clique.
4. **Engine de Layout Torin:** Cálculo exato de retângulos em pixels, aplicação de padding, gap entre elementos e distribuição direcional.
5. **Dimensionamento Percentual Torin:** Resolução recursiva de larguras e alturas percentuais (`50%`, `25%`) em relação à área útil do elemento pai.
6. **Hit-Testing e Geometria:** Validação geométrica de contenção de ponto (`contains(x, y)`) em retângulos delimitadores.

### Como Executar os Testes

```bash
cargo test -p aipo-game-host --test bridge_tests test_freya_ui_unit_test_suite
```
