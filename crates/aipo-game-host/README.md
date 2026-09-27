# aipo-game-host

Host nativo da engine de jogos **Aipo** construído em Rust sobre o ecossistema **Miniquad / Macroquad**.

Fornece:
- **Janela acelerada por hardware:** Renderização 2D nativa a 60 FPS cravados via OpenGL / GLX sem overhead de frameworks pesados.
- **Execução Dinâmica de Scripts `.aipo`:** Carrega, compila e roda arquivos `.aipo` em tempo real diretamente através da VM do Aipo (`aipo-vm`).
- **Hot-Reload em Tempo Real:** Pressione `F5` ou `Ctrl+R` para recompilar e atualizar o script sem fechar a janela.
- **Resiliência Headless:** Todas as chamadas Macroquad possuem fallback seguro sem panics para testes automatizados em ambientes de CI.
- **Demo Interativa Embutida:** Executar sem argumentos abre instantaneamente o Snake Game interativo a 60 FPS.

## Como Usar

### 1. Rodar um script de jogo `.aipo`
```bash
cargo run -p aipo-game-host -- examples/26_interactive_game.aipo
```
ou utilizando o binário compilado:
```bash
./target/debug/aipo-game-host examples/26_interactive_game.aipo
```

### 2. Rodar a demonstração embutida
```bash
cargo run -p aipo-game-host
# ou
./target/debug/aipo-game-host
```

## Interface FFI do Host (`host_bridge`)

Os scripts `.aipo` têm acesso nativo e sem atrito a:

| Função | Parâmetros | Descrição |
|---|---|---|
| `host_clear_background(r, g, b)` | `Float, Float, Float` | Limpa a tela com a cor RGB informada |
| `host_draw_rect(x, y, w, h, r, g, b, a)` | `Float...` | Desenha um retângulo preenchido na GPU |
| `host_draw_rect_lines(x, y, w, h, th, r, g, b, a)` | `Float...` | Desenha as bordas de um retângulo |
| `host_draw_circle(cx, cy, radius, r, g, b, a)` | `Float...` | Desenha um círculo preenchido |
| `host_draw_text(text, x, y, size, r, g, b)` | `String, Float...` | Renderiza texto na tela |
| `host_load_texture(path)` | `String` | Carrega textura PNG/JPEG do disco (retorna ID) |
| `host_draw_sprite(id, x, y, w, h, rot, flip_x)` | `Int, Float...` | Renderiza sprite com rotação e espelhamento |
| `host_draw_sprite_subrect(id, sx, sy, sw, sh, dx, dy, dw, dh, flip_x)` | `Int, Float...` | Renderiza recorte de spritesheet |
| `host_key_down(key_code)` | `Int` | Retorna `true` se a tecla estiver pressionada |
| `host_key_pressed(key_code)` | `Int` | Retorna `true` no frame exato em que a tecla foi acionada |
| `host_mouse_x()`, `host_mouse_y()` | Nenhum | Retorna as coordenadas X e Y do mouse |
| `host_mouse_btn(button)` | `Int` | Retorna o estado do botão (0=esq, 1=dir, 2=meio) |
| `host_mouse_wheel_x()`, `host_mouse_wheel_y()` | Nenhum | Retorna o deslocamento da roda de rolagem do mouse |
| `host_push_clip_rect(x, y, w, h)` | `Float, Float, Float, Float` | Empilha retângulo de recorte para corte de GPU em 2D |
| `host_pop_clip_rect()` | Nenhum | Desempilha o retângulo de recorte ativo |
| `host_screen_width()`, `host_screen_height()` | Nenhum | Retorna as dimensões atuais da janela |
| `host_frame_time()` | Nenhum | Retorna o delta time `dt` em segundos (~0.016s a 60 FPS) |
| `host_set_camera(tx, ty, zoom)` | `Float, Float, Float` | Move e aplica zoom na câmera 2D |
| `host_reset_camera()` | Nenhum | Retorna a câmera para a projeção padrão |
| `host_load_sound(path)` | `String -> Int` | Carrega arquivo de áudio WAV/OGG em memória e retorna ID |
| `host_play_sound(id, volume, pitch)` | `Int, Float, Float` | Reproduz som por ID com volume (0.0 a 1.0) e pitch |
| `host_play_preset(name, volume, pitch)` | `String, Float, Float` | Toca SFX chiptune procedural instantâneo ("coin", "laser", "jump", "explosion", "hit", "powerup", "click") |
| `host_synth_sound(wave, freq, slide, dur, vol)` | `String, Float... -> Int` | Sintetiza forma de onda procedural em tempo real gerando WAV 16-bit |
| `host_stop_sound(id)` | `Int` | Interrompe o som especificado por ID |
| `host_play_music(id, volume, loop)` | `Int, Float, Bool` | Toca trilha musical em loop |
| `host_stop_music()` | Nenhum | Interrompe a música de fundo ativa |

Todas as funções também estão disponíveis através do módulo global `game` (ex: `game.draw_rect(...)`, `game.play_preset(...)`) e pelos aliases canônicos `__aipo_game_*`.
