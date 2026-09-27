# Exemplos de aipo.game

Demonstrações práticas e executáveis do ecossistema `aipo.game` rodando em desktop nativo a 60 FPS com aceleração por hardware via Miniquad/Macroquad no `aipo-game-host`.

## Exemplos Disponíveis

### 1. `tilemap_and_hud.aipo`
Demonstra a integração completa dos subsistemas de **Tilemap 2D com Resolução de Colisão AABB** e **HUD de Jogo em Modo Imediato**:
- **Grid Tilemap 2D**: mapa 24x18 com blocos 32x32 e limites de arena sólidos.
- **Resolução de Colisão Eixo por Eixo (Swept AABB)**: o jogador move-se em alta velocidade deslizando suavemente nas paredes sem atravessar obstáculos ou apresentar engasgos.
- **Câmera 2D Suave e Zoom Dinâmico**: a câmera persegue a posição do jogador suavemente; zoom interativo com teclas `Z` e `X`.
- **DDA Grid Raycasting**: traçado de raio em tempo real do centro do jogador até a posição do cursor do mouse, com detecção precisa de impacto em células sólidas e renderização de linha de visada / mira laser.
- **HUD Imediato do Jogo**:
  - Barra de vida dinâmica com gradação de cor (verde / amarelo / vermelho).
  - Barra de estamina com recarga contínua.
  - Painel de controle no canto superior com botão de debug para visualização da malha do grid.
  - Placar interativo com contadores de quadros e botões clicáveis.

#### Como Executar
```bash
cargo run -p aipo-game-host -- packages/aipo-game/examples/tilemap_and_hud.aipo
```

#### Controles
- **WASD / Setas**: Movimentação do jogador com aceleração e deslizamento em quinas.
- **Z / X**: Zoom in / Zoom out da câmera virtual.
- **Mouse**: Mira laser com raycasting DDA e clique em botões do HUD.
- **Botão "Toggle Grid"**: Alterna exibição das linhas da grade de colisão do tilemap.

---

### 2. `animation_and_particles.aipo`
Demonstra a máquina de estados para **Animação de Sprites 2D** e o **Sistema de Partículas 2D com Física**:
- **Máquina de Estados de Animação**:
  - Clipes `idle`, `run` e `attack` com taxas de quadros (FPS) dedicadas e controle de looping.
  - Espelhamento horizontal automático (`flip_x`) conforme a direção de movimento do jogador.
  - Transição de retorno a `idle`/`run` após o término de ações não-contínuas (*one-shot*).
- **Sistema de Partículas com Física e Presets**:
  - Efeito de poeira nos pés (`dust`) gerado dinamicamente durante a corrida.
  - Explosões pirotécnicas (`explosion`) e faíscas brilhantes (`sparks`) disparadas no ataque.
  - Efeitos interativos adicionais com chuva de moedas douradas (`coins`) e pluma de fumaça (`smoke`).
  - Dinâmica com gravidade, arrasto/resistência e atenuação suave de transparência (*alpha fading*).
- **HUD Diagnóstico Integrado**:
  - Painel imediato exibindo clipe ativo, índice do quadro atual e contagem total de partículas ativas a 60 FPS.

#### Como Executar
```bash
cargo run -p aipo-game-host -- packages/aipo-game/examples/animation_and_particles.aipo
```

#### Controles
- **WASD / Setas**: Movimentação do ator com troca automática de animação para `run` e flip direcional.
- **Espaço / Botão Esquerdo do Mouse**: Ataque com espada, acionando animação `attack`, faíscas e explosão.
- **E**: Disparo de chuva de partículas de moedas douradas (`coins`).
- **Q**: Disparo de pluma de fumaça ascendente com expansão de raio (`smoke`).

