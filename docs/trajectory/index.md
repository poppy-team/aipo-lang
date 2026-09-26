# Trajetória de Desenvolvimento

Esta seção documenta a jornada completa de engenharia, arquitetura e evolução da linguagem **Aipo**, desde a concepção inicial dos primeiros tokens até a consolidação da versão v0.1.0.

Cada onda (*wave*) e fatia vertical (*slice*) foi construída sob a metodologia **Lean Progressive Context (LPC)** e governada pelo **Prumo CLI**, com critérios rigorosos de aceitação, testes exaustivos e zero suposições não comprovadas.

---

## Linha do Tempo das Ondas de Implementação

```mermaid
timeline
    title Trajetória de Engenharia da Linguagem Aipo
    Wave 0 (MVP) : S1 ao S3 (Lexer, Parser, AST)
                 : S4 ao S6 (HIR, Sema, IR, Bytecode)
                 : S7 ao S11 (VM, Stdlib, CLI, Conformance)
    Wave 1 (Contratos) : Hooks de Construção (init)
                       : Invariantes com Rollback (attempt)
                       : Interfaces Estruturais & Funções Locais
    Wave 2 (JS Parity) : Emissor aipo-js & Shim ES2022
                       : Paridade Diferencial VM x JS
                       : Fuzzing & Auditoria cargo deny
    Wave 3 (Async) : Tipos Ricos (Set, Sequence, Bytes)
                   : async fn & await do sequencial
                   : Scheduler Determinístico com Tempo Virtual
    Wave 4 (Host ABI) : Crate aipo-host com Capabilities em Árvore
                      : Handles Geracionais Anti Use-After-Free
                      : Adaptador Poppy e Simulação Headless
    Wave 5 (Pacotes) : Manifesto aipo.toml & Lockfile Canônico
                     : Dependências GitHub Pinadas por Commit SHA
                     : Cache Local Verificável com SHA-256
    Wave 6 (v0.1.0) : Delimitação da Release v0.1.0 (ADP-008)
                    : C ABI Síncrona Versionada (ADP-009)
                    : Thin Proofs Rust, C e JS (ADP-010)
```

---

## Sumário das Ondas de Implementação

<div class="wave-grid">

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 0 — MVP da Linguagem</h4>
<p>Implementação dos 11 slices verticais fundamentais: Lexer UTF-8, Parser resiliente, HIR, SEMA, Core IR, Bytecode, VM de pilha/registradores, Stdlib mínima, Formatter, CLI unificada e suíte inicial de conformance.</p>
<p><a href="/trajectory/wave-0-mvp">Ler documentação da Wave 0 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 1 — Contratos & Conformance</h4>
<p>Fechamento do MVP de linguagem com hooks <code>init()</code>, invariantes de integridade <code>invariant()</code> em fronteiras estáveis de mutação com rollback automático via journal transacional e conformidade de interfaces.</p>
<p><a href="/trajectory/wave-1-contracts">Ler documentação da Wave 1 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 2 — Paridade JS & Qualidade</h4>
<p>Criação do emissor <code>aipo-js</code> para JavaScript moderno (ES2022) com runtime shim modular, suíte de conformance diferencial VM↔JS, testes de propriedade, fuzzing contínuo e política estrita de dependências com <code>cargo deny</code>.</p>
<p><a href="/trajectory/wave-2-js-parity">Ler documentação da Wave 2 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 3 — Tipos Ricos & Async</h4>
<p>Introdução de <code>Set</code> com ordem de inserção, <code>Sequence</code> lazy, empacotamento de <code>Bytes</code>, sintaxe nativa <code>async fn</code> e <code>await do</code>, combinadores assíncronos e scheduler cooperativo com tempo virtual.</p>
<p><a href="/trajectory/wave-3-async">Ler documentação da Wave 3 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 4 — Host ABI & Poppy Engine</h4>
<p>Design da Host ABI segura (<code>aipo-host</code>) com permissões deny-by-default, handles geracionais resistentes a use-after-free, barreira contra escape de escopo e integração com o motor headless de simulação Poppy.</p>
<p><a href="/trajectory/wave-4-host-poppy">Ler documentação da Wave 4 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Concluído</span>
<h4>Wave 5 — Pacotes Herméticos</h4>
<p>Sistema de gerenciamento de dependências e distribuição de pacotes hermético: coordenadas <code>namespace.package</code>, dependências GitHub pinadas por commit SHA com digest verificado, cache local atômico e replay 100% offline.</p>
<p><a href="/trajectory/wave-5-packages">Ler documentação da Wave 5 →</a></p>
</div>

<div class="wave-card">
<span class="milestone-badge">Em Curso</span>
<h4>Wave 6 — Release v0.1.0</h4>
<p>Formalização da fronteira canônica do produto (ADP-008): foco exclusivo na linguagem e seu ferramental autônomo, test runner minimalista, C ABI síncrona versionada (ADP-009) e provas finas de interoperabilidade em Rust, C e JavaScript (ADP-010).</p>
<p><a href="/trajectory/wave-6-release-v010">Ler documentação da Wave 6 →</a></p>
</div>

</div>
