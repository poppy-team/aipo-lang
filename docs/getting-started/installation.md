# Instalação & Setup

O compilador e a suíte de ferramentas do **Aipo** estão distribuídos como um workspace unificado em Rust.

---

## Pré-requisitos

Para compilar e executar o Aipo em sua máquina:

- **Rust 1.85+** (MSRV verificado com suporte a Rust 2024 edition)
- **Cargo** (incluso com o Rust toolchain)
- **Node.js 18+** *(opcional, necessário apenas se for utilizar o backend `aipo-js` para executar via Node)*

Para instalar o Rust em distribuições Linux, macOS ou WSL:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Compilação a partir do Código-Fonte

Clone o repositório do Aipo e compile o binário da interface de linha de comando (`aipo-cli`):

```bash
# 1. Clonar o repositório
git clone https://github.com/poppy-team/aipo-lang.git
cd aipo-lang

# 2. Compilar em modo release com otimizações máximas
cargo build --release -p aipo-cli

# 3. O binário executável estará disponível em:
./target/release/aipo --version
```

### Adicionando ao seu `PATH`

Para disponibilizar o comando `aipo` globalmente no seu sistema:

```bash
# No Linux / macOS
cp ./target/release/aipo ~/.local/bin/

# Verifique a instalação
aipo --help
```

---

## Verificação da Instalação

Após a instalação, verifique se todos os subsistemas estão funcionais:

```bash
aipo check --help
aipo run --help
aipo disasm --help
```

Para validar a integridade completa do workspace e executar a suíte com mais de 500 testes:

```bash
cargo test --workspace
```
