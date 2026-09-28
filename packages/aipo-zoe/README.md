# Zoe UI (`aipo.zoe`)

Framework declarativo de interface de usuário (GUI) para a linguagem **Aipo**, baseado no algoritmo de layout **Leona**, sistema reativo de hooks (`use_state`) e componentes visuais estilizados na paleta **Catppuccin Mocha**.

O **Zoe UI** foi concebido para o ecossistema Aipo com arquitetura pura, declarativa e leve: o código roda diretamente na máquina virtual Aipo e renderiza em GPU nativa (60+ FPS) via [`aipo-game-host`](file:///home/raillen/Documentos/Projetos/aipo-lang/crates/aipo-game-host), com compatibilidade pronta para futuros alvos WebAssembly e JavaScript.

---

## Características Principais

1. **Árvore Declarativa e Containers Ergonômicos:**
   Construção de interfaces limpas e expressivas com containers sem cerimônia (`column`, `row`, `stack`, `center`, `spacer`, `divider`, `split_view`, `tab_bar`, `tab_view`, `scroll_view`, `viewport`) e catálogo completo de componentes (`button`, `switch`, `slider`, `text_input`, `number_input`, `dropdown_select`, `color_picker`, `card`, `badge`, `label`, `rect`).

2. **Engine de Layout Leona:**
   Algoritmo hierárquico de medição e posicionamento que resolve:
   - Dimensões absolutas em pixels (`Float` / `Int`).
   - Dimensões relativas em porcentagem (`"100%"`, `"50%"`).
   - Dimensões intrínsecas e dimensionamento flexível (`"flex"`, `"auto"`).
   - Alinhamento transversal (`align_items`: `"start"`, `"center"`, `"end"`).
   - Distribuição no eixo principal (`justify_content`: `"start"`, `"center"`, `"end"`).
   - Espaçamento interno (`padding`, `padding_x`, `padding_y`) e espaçamento entre filhos (`gap`).

3. **Reatividade Fina com Hooks:**
   Runtime de sinais reativos (`use_state`, `set_state`) com rastreamento automático de quadros sujos (*dirty flags*), disparando reconstrução e re-layout apenas quando o estado sofre mutação.

4. **Catppuccin Mocha Embutido:**
   Paleta completa de cores modernas acessíveis via `zoe.color` (`zoe.color.base`, `zoe.color.blue`, `zoe.color.mantle`, etc.) e utilitários `rgb`, `rgba`, `hex`.

5. **Aceleração por GPU Nativa:**
   Renderização com 60+ FPS no desktop nativo conectada diretamente ao host Miniquad através do `aipo-game-host`.

6. **Ferramentas de Viewport & Edição Visual (Picking e Gizmos):**
   Suporte a sub-retângulo GPU Scissor e câmera 2D (`set_viewport_camera`), transformações bidirecionais de tela/mundo (`world_to_screen`, `screen_to_world`), detecção espacial AABB (`point_in_rect`), hit-testing de manipuladores (`test_gizmo_hit`) e renderização de Gizmos de translação 2D (`draw_gizmo_2d`) com restrição de eixos (X, Y e Centro livre).

7. **Motor 3D Retro Completo (`retro3d`):**
   Renderização 3D completa com estética retro sem sobrecarga de pipelines pesados:
   - **Matemática Vetorial 3D:** Funções nativas `vec3`, `vec3_add`, `vec3_sub`, `vec3_scale`, produto escalar/vetorial e normalização.
   - **Câmera Orbital Perspectiva:** `camera_3d` e `project_3d` com rotação `yaw` (horizontal) e `pitch` (vertical), distância orbital e *near-plane clipping*.
   - **Primitivas de Malha:** Criação de malhas tridimensionais com vetores normais (`cube_mesh`, `pyramid_mesh`).
   - **Grade de Chão Infinita:** `draw_grid_3d` com linhas convergentes ao horizonte.
   - **Rasterizador Sólido com Flat-Shading:** `render_mesh_3d` com iluminação difusa direcional (Lambertian), descarte de faces traseiras (*backface culling*) em screen space e ordenação de triângulos pelo Algoritmo do Pintor (*Painter's depth sorting*).
   - **Gizmo 3D e Picking:** Manipulador de 3 eixos (`draw_gizmo_3d`, `test_gizmo_3d_hit`) com X (Pêssego), Y (Verde), Z (Azul) e cubo central para manipulação direta de entidades 3D no espaço.

---

## Instalação

Adicione a dependência ao seu `aipo.toml`:

```toml
[dependencies]
"aipo.zoe" = { path = "packages/aipo-zoe" }
```

Gere ou audite o lockfile do seu projeto:

```bash
aipo package lock .
aipo package audit .
```

---

## Exemplo Rápido: Contador Reativo Ergonômico

```aipo
import aipo.zoe as zoe

fn view() {
    let count = zoe.use_state(0)

    return zoe.center({ "background": zoe.color.base, "gap": 16.0 }, [
        zoe.label(f"Contador: {count.get()}", {
            "font_size": 24.0,
            "color": zoe.color.text
        }),
        zoe.button("Incrementar +1", _ => zoe.set_state(count, count.get() + 1), {
            "variant": "primary",
            "background": zoe.color.blue
        })
    ])
}

fn setup() {
    zoe.mount(view)
}

fn update(dt) {
    zoe.step(dt)
}

fn draw() {
    zoe.draw_ui()
}
```

---

## Executando Exemplos de Demonstração

- Um dashboard completo e interativo com contador, switches, sliders e barra de progresso está disponível em `examples/dashboard.aipo`:
```bash
cargo run -p aipo-game-host -- packages/aipo-zoe/examples/dashboard.aipo
```

- Um editor visual de game engine completo com alternância de modos **2D e 3D**, splitters redimensionáveis, abas de hierarquia e assets, viewport com picking de objetos, manipulador de transformação visual (Gizmo 2D e Gizmo 3D de 3 eixos), câmeras interativas (Pan/Zoom 2D e Órbita/Dolly Zoom 3D), renderização retro 3D com iluminação flat e inspetor com edição bidirecional em tempo real (coordenadas X, Y e Z) está disponível em `examples/editor.aipo`:
```bash
cargo run -p aipo-game-host -- packages/aipo-zoe/examples/editor.aipo
```

---

## Testes Automatizados

O pacote conta com uma suíte de testes de unidade e integração:

```bash
# Executa a suíte de testes do framework Zoe UI
cargo test -p aipo-game-host --test bridge_tests test_zoe_ui_unit_test_suite

# Executa o teste de compilação e execução estável multi-frame do Dashboard
cargo test -p aipo-game-host --test bridge_tests test_zoe_ui_dashboard_compilation_and_execution

# Executa o teste de compilação e execução estável multi-frame do Editor Visual
cargo test -p aipo-game-host --test bridge_tests test_zoe_ui_editor_compilation_and_execution
```
