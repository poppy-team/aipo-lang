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
