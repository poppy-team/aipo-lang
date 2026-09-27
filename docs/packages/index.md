# Pacotes & Frameworks Oficiais

O ecossistema oficial da linguagem **Aipo** é composto por bibliotecas e frameworks de alto nível projetados com os mesmos padrões de rigor, segurança por padrão e determinismo do compilador central.

Conforme estabelecido pela governança da linguagem, **recursos de domínio específico (como HTML, CSS, UI, simulações e áudio) não inflam a gramática do compilador**. Em vez disso, eles são distribuídos como pacotes canônicos gerenciados pelo `aipo package`.

---

## Catálogo de Pacotes Oficiais

| Pacote | Versão | Domínio | Descrição |
|---|:---:|---|---|
| **[`aipo.html`](/packages/aipo-html)** | `v0.1.0` | **Web & DOM** | DSL declarativa para HTML5, CSS-in-Aipo tipado e reatividade com **MVU de Granularidade Fina** no navegador. |
| **[`aipo.ui`](/packages/aipo-ui)** | `v0.1.0` | **UI Multiplataforma** | Framework declarativo de interface universal (Desktop GPU via Skia/Freya, WebGL e Terminal TUI) com layout Flexbox/Taffy. |
| **[`aipo.game`](/packages/aipo-game)** | `v0.1.0` | **Game Engine 2D** | Micro-engine 2D orientada a Atores e Cenas com comportamentos em 1 linha, nós visuais e determinismo de simulação. |
| **[`aipo.http`](/packages/aipo-http)** *(Roadmap)* | `v0.1.0-alpha` | **Web Server & APIs** | Microframework HTTP/WebSocket assíncrono com roteamento Radix tree tipo-seguro inspirado em Hono e FastAPI. |

---

## Como Utilizar um Pacote Oficial

No arquivo `aipo.toml` do seu projeto, declare a dependência sob a tabela `[dependencies]`:

```toml
[package]
name = "meu-projeto"
version = "0.1.0"

[dependencies]
"aipo.html" = { path = "packages/aipo-html" }
```

Em seguida, no seu código `.aipo`:

```aipo
import aipo.html as h

h.div(class: "app") {
    h.h1("Construído com Aipo!")
}
```
