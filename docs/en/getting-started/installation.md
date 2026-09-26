# Installation & Setup

The Aipo compiler and toolchain are distributed as an open unified workspace in Rust.

---

## Prerequisites

To build and run Aipo locally:

- **Rust 1.85+** (MSRV verified with Rust 2024 edition support)
- **Cargo** (included with the standard Rust toolchain)
- **Node.js 18+** *(optional, only needed when executing output emitted by `aipo-js`)*

To install Rust on Linux, macOS, or WSL:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Building from Source

Clone the Aipo repository and compile the command-line interface binary (`aipo-cli`):

```bash
# 1. Clone the repository
git clone https://github.com/poppy-team/aipo-lang.git
cd aipo-lang

# 2. Build in release mode with maximum optimization
cargo build --release -p aipo-cli

# 3. The executable binary will be available at:
./target/release/aipo --version
```

### Adding to your `PATH`

To make the `aipo` command accessible globally:

```bash
# On Linux / macOS
cp ./target/release/aipo ~/.local/bin/

# Verify installation
aipo --help
```

---

## Verifying Installation

Verify that all subsystems operate properly:

```bash
aipo check --help
aipo run --help
aipo disasm --help
```

To validate the workspace integrity and execute the full suite of 500+ automated tests:

```bash
cargo test --workspace
```
