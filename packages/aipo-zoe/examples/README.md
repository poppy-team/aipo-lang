# Exemplos do aipo.zoe (`examples/`)

Este diretório contém aplicações de demonstração e vitrines do framework declarativo `aipo.zoe` (Zoe UI).

---

## Exemplos Disponíveis

### `dashboard.aipo`
Dashboard interativo com estilo Catppuccin Mocha apresentando:
- **Contador Reativo:** Botões de incremento (+1), decremento (-1) e reset vinculados ao hook `use_state`.
- **Controle de Sistema (Switch):** Alternador On/Off de modo turbo GPU demonstrando alternância de estado booleano e animação de knob.
- **Controle de Volume (Slider & Progress Bar):** Barra de progresso visual com botões de preset (25%, 50%, 75%, 100%) mutando o estado e disparando re-renderização em tempo real.
- **Layout Leona Complexo:** Cards, badges com bordas arredondadas, tipografia com peso bold, e distribuição flexível de colunas e linhas.

### `editor.aipo`
Editor visual completo de game engine com arquitetura modular demonstrando:
- **Painéis Redimensionáveis (`split_view`):** Divisões horizontais interativas entre árvore de nós, viewport central e inspetor lateral.
- **Abas Organizadoras (`tab_view`):** Alternância entre hierarquia de entidades e catálogo de assets.
- **Viewport Integrada com Câmera GPU (`viewport`):** Renderização direta da cena 2D com scissor clipping, navegação contínua com arrasto (`on_drag`) e zoom com roda de rolagem (`on_wheel`).
- **Inspetor Reativo:** Seleção interativa de entidades na árvore refletida instantaneamente no painel de propriedades.

### Como Executar

```bash
# Executar o Dashboard
cargo run -p aipo-game-host -- packages/aipo-zoe/examples/dashboard.aipo

# Executar o Editor Visual
cargo run -p aipo-game-host -- packages/aipo-zoe/examples/editor.aipo
```
