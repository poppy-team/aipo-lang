# aipo-egui

`aipo-egui` é o adaptador host agnóstico e bindings da biblioteca Immediate-Mode GUI **egui** para a linguagem Aipo, implementando a prova de interoperabilidade Rust especificada em [ADP-010](../../docs/adp/ADP-010-interoperability-thin-proofs.md).

---

## Arquitetura & Garantias

1. **Totalmente Agnóstico de Engine / Plataforma:**
   - O núcleo opera estritamente sobre a máquina de estados pura do `egui::Context`.
   - **Zero dependências** de janelas de sistema operacional (sem `winit`), drivers gráficos ou motores de jogos (sem acoplamento a Macroquad/Miniquad).
   - Funciona em qualquer renderizador consumidor: GPU nativo, WebAssembly Canvas, Terminal TUI ou testes headless automatizados.

2. **AHS Host Surface (`egui_schema`)**:
   - Descrito como dados via Aipo Host Schema (AHS).
   - Expõe o módulo `egui`, handles geracionais `Context`, capability `egui`, e funções nativas:
     - Ciclo de vida: `create_context`, `destroy_context`, `begin_frame`, `end_frame`.
     - Contêineres: `begin_window`, `end_window`.
     - Widgets imediatos: `label`, `heading`, `separator`, `button`, `checkbox`, `slider`, `text_edit`, `progress_bar`.
     - Foco e estado: `wants_pointer_input`, `wants_keyboard_input`.

3. **Handles Geracionais Seguros (`HandleTable`)**:
   - Sessões egui são acessadas exclusivamente através de handles geracionais opacos de `aipo-host`.
   - Chamar `destroy_context` avança a geração do slot; tentativas posteriores de reuso emitem `AIPO_RT_STALE_HANDLE` (nunca *use-after-free* ou *panic*).

4. **Modelo de Segurança Capability Sandbox (`deny-by-default`)**:
   - Qualquer operação egui exige a permissão `egui`.
   - Sem a capability concedida ao ambiente do VM, a execução emite imediatamente `AIPO_RT_CAPABILITY_DENIED`.

5. **Extração de Primitivas Geométricas Agnósticas (`shapes`)**:
   - O método `end_frame` converte a saída gráfica do `egui::FullOutput` em uma lista de dicionários (`Value::Dict`) universais:
     - `rect`: coordenadas `[min_x, min_y, max_x, max_y]`, cor de preenchimento `fill`, borda `stroke_color`, espessura `stroke_width` e raio dos cantos `corner_radius`.
     - `text`: texto bruto, posição `[x, y]`, tamanho `[width, height]` e cor.
     - `circle`, `line`, `bezier`, `path`.
   - Qualquer backend de desenho pode iterar e rasterizar essa lista com zero acoplamento ao egui.

6. **Código 100% Seguro:**
   - `#![forbid(unsafe_code)]` ativado e rigorosamente cumprido em todos os arquivos da crate.

---

## Testes

Execute a suíte completa de testes unitários e de integração:
```bash
cargo test -p aipo-egui
```
Cobre validação do schema AHS, ciclo de vida e handles geracionais, checagem de capabilities, extração de shapes e execução de scripts `.aipo` de ponta a ponta.
