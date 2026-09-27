# Arquitetura Interna do aipo.zoe (`src/`)

Este diretório contém a implementação modular do framework Zoe UI para a linguagem Aipo.

---

## Módulos

| Módulo | Responsabilidade |
| :--- | :--- |
| `types.aipo` | Estruturas fundamentais (`Rect`, `Color`, `Element`, `StateSignal`) e construtores de domínio. |
| `color.aipo` | Normalização de cores para shader/GPU (`to_rgb`, `to_rgba`), parser hexadecimal (`parse_hex`) e paleta Catppuccin Mocha completa. |
| `torin.aipo` | Engine de layout Torin para Aipo: cálculo de caixas, unidades fixas/percentuais/flex, alinhamento (`start`/`center`/`end`) e distribuição (`justify_content`). |
| `hooks.aipo` | Runtime de reatividade por hooks (`signal_use_state`, `signal_set_value`) com controle de quadros alterados (*dirty frames*). |
| `elements.aipo` | Primitivas base da árvore visual (`make_rect`, `make_label`, `make_container`). |
| `components.aipo` | Catálogo de componentes de alto nível (`make_button`, `make_switch`, `make_slider`, `make_progress_bar`, `make_card`, `make_badge`). |
| `renderer.aipo` | Passada de renderização que percorre a árvore resolvida pelo Torin e despacha chamadas nativas de GPU (`host_draw_rect`, `host_draw_rect_lines`, `host_draw_text`). |
| `app.aipo` | Ciclo de vida da aplicação (`app_mount`, `app_update`, `app_render`), hit-testing geométrico e despacho de eventos de clique. |
| `lib.aipo` | Ponto de entrada canônico do pacote, expondo a API pública estável sem colisões de escopo. |

---

## Fluxo de Execução

1. **Mount (`mount(view)`):** Registra a função geradora da árvore e instancia os hooks de estado inicial.
2. **Step (`step(dt)`):**
   - Amostra as coordenadas e botões do mouse do host nativo.
   - Efetua hit-testing contra a geometria computada na árvore visual.
   - Despacha callbacks interativos (`on_click`) quando um botão é acionado.
   - Se o estado mudou (`is_dirty()`), reconstrói a árvore de elementos e executa uma nova passada da engine de layout Torin.
3. **Render (`draw_ui()`):**
   - Limpa o fundo com a cor de tema configurada.
   - Caminha recursivamente pela árvore Torin desenhando retângulos, bordas, textos e efeitos visuais de hover.
