# Exemplos do aipo.zoe (`examples/`)

Este diretório contém aplicações de demonstração e vitrines do framework declarativo `aipo.zoe` (Zoe UI).

---

## Exemplos Disponíveis

### `dashboard.aipo`
Dashboard interativo com estilo Catppuccin Mocha apresentando:
- **Contador Reativo:** Botões de incremento (+1), decremento (-1) e reset vinculados ao hook `use_state`.
- **Controle de Sistema (Switch):** Alternador On/Off de modo turbo GPU demonstrando alternância de estado booleano e animação de knob.
- **Controle de Volume (Slider & Progress Bar):** Barra de progresso visual com botões de preset (25%, 50%, 75%, 100%) mutando o estado e disparando re-renderização em tempo real.
- **Layout Torin Complexo:** Cards, badges com bordas arredondadas, tipografia com peso bold, e distribuição flexível de colunas e linhas.

### Como Executar

```bash
cargo run -p aipo-game-host -- packages/aipo-zoe/examples/dashboard.aipo
```
