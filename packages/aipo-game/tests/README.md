# Suíte de Testes do aipo.game (`tests/`)

Este diretório contém os testes de unidade automatizados do framework `aipo.game`.

---

## Arquivos de Teste

### `game_test.aipo`
Suíte de testes de unidade cobrindo:
1. **Clip de Animação e Defaults:** Verificação de integridade dos campos de `AnimationClip` (nome, contagem de quadros, FPS seguro, dimensões de recorte de célula e colunas do tileset).
2. **Registro e Troca de Clipes no Animador:** Registro de múltiplos clipes em `SpriteAnimator`, seleção de clipe inicial, disparo de troca de clipes via `play_animation` e verificação de reinício de temporizador e índice de quadro.
3. **Avanço Temporal e Término de Clipes One-Shot:** Simulação de avanço de tempo (`update_animator`), transição quadro-a-quadro e travamento no último quadro com sinalizador `is_finished` ativado em clipes não-contínuos (`is_looping == false`).
4. **Emissor de Partículas e Gerenciamento de Memória:** Alocação de `ParticleEmitter`, respeito ao teto de capacidade `max_particles`, reposicionamento espacial e limpeza instantânea (`clear_particles`).
5. **Física e Decaimento de Partículas:** Aplicação de velocidade linear, gravidade, fator de arrasto e descarte determinístico de partículas expiradas (`life <= 0.0`).
6. **Despacho de Presets Visuais:** Verificação dos geradores parametrizados de partículas (`"sparks"`, `"explosion"`, `"smoke"`, `"dust"`, `"coins"`).

---

### Como Executar os Testes

```bash
cargo test -p aipo-game-host --test bridge_tests test_game_subsystems_unit_test_suite
```
