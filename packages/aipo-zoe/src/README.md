# Arquitetura Interna do aipo.zoe (`src/`)

Este diretório contém a implementação modular do framework Zoe UI para a linguagem Aipo.

---

## Módulos

| Módulo | Responsabilidade |
| :--- | :--- |
| `types.aipo` | Estruturas fundamentais (`Rect`, `Color`, `Element`, `StateSignal`) e construtores de domínio. |
| `color.aipo` | Normalização de cores para shader/GPU (`to_rgb`, `to_rgba`), parser hexadecimal (`parse_hex`) e paleta Catppuccin Mocha completa. |
| `leona.aipo` | Engine de layout Leona para Aipo: cálculo de caixas, unidades fixas/percentuais/flex, alinhamento (`start`/`center`/`end`) e distribuição (`justify_content`). |
| `hooks.aipo` | Runtime de reatividade por hooks (`signal_use_state`, `signal_set_value`) com controle de quadros alterados (*dirty frames*). |
| `elements.aipo` | Primitivas base da árvore visual (`make_rect`, `make_label`, `make_container`). |
| `components.aipo` | Catálogo de componentes de alto nível (`make_button`, `make_switch`, `make_slider`, `make_progress_bar`, `make_card`, `make_badge`, `make_split_view`, `make_tab_bar`, `make_tab_view`, `make_scroll_view`). |
| `renderer.aipo` | Passada de renderização que percorre a árvore resolvida pela Leona e despacha chamadas nativas de GPU (`host_draw_rect`, `host_draw_rect_lines`, `host_draw_text`), com realce de hover/active e suporte a recorte de GPU (`clip`). |
| `app.aipo` | Ciclo de vida da aplicação (`app_mount`, `app_update`, `app_render`), hit-testing geométrico, eventos de clique, captura contínua de arraste (`on_drag`) e despacho de rolagem de roda (`on_wheel`). |
| `lib.aipo` | Ponto de entrada canônico do pacote, expondo a API pública estável com containers ergonômicos (`column`, `row`, `stack`, `center`, `spacer`, `divider`, `split_view`, `tab_bar`, `tab_view`, `scroll_view`). |

---

## Fluxo de Execução

1. **Mount (`mount(view)`):** Registra a função geradora da árvore e instancia os hooks de estado inicial.
2. **Step (`step(dt)`):**
   - Amostra as coordenadas e botões do mouse do host nativo.
   - Efetua hit-testing contra a geometria computada na árvore visual.
   - Despacha callbacks interativos de clique (`on_click`) no limiar de clique.
   - Gerencia captura de ponteiro para arraste contínuo (`on_drag`, `on_drag_end`) em elementos interativos e divisores de tela.
   - Se o estado mudou (`is_dirty()`), reconstrói a árvore de elementos e executa uma nova passada da engine de layout Leona.
3. **Render (`draw_ui()`):**
   - Limpa o fundo com a cor de tema configurada.
   - Caminha recursivamente pela árvore resolvida desenhando retângulos, bordas, textos e efeitos visuais de hover e active.
