# Guia do Utilitário de Linha de Comando (`aipo`)

O executável `aipo` é a ferramenta unificada para execução, análise estática, compilação, formatação e gerenciamento de pacotes da linguagem.

---

## Subcomandos Principais

### `aipo run`

Executa um arquivo de código-fonte (`.aipo`) ou um bytecode binário pré-compilado (`.aibc`):

```bash
# Executar código-fonte diretamente
aipo run main.aipo

# Executar arquivo de bytecode compilado
aipo run app.aibc

# Executar com argumentos
aipo run script.aipo -- --flag valor
```

### `aipo check`

Executa a análise estática completa (Lexer, Parser, HIR e Semântica) sem gerar bytecode nem executar a VM. Ideal para CI e editores:

```bash
aipo check src/main.aipo
```

Reporta erros de sintaxe, contratos de interface incompatíveis, referências a variáveis indefinidas ou mutações ilegais de campos `fixed`.

### `aipo build`

Compila arquivos `.aipo` para os alvos suportados:

```bash
# Compilar para bytecode binário (.aibc)
aipo build src/main.aipo -o dist/main.aibc

# Compilar para JavaScript moderno (.js)
aipo build src/main.aipo -t js -o dist/bundle.js
```

### `aipo disasm`

Desassembla arquivos de código ou arquivos binários `.aibc`, exibindo as instruções da VM, offsets, constantes e linhas de código originais:

```bash
aipo disasm src/main.aipo
```

### `aipo fmt`

Formata arquivos de código Aipo de acordo com os padrões canônicos da linguagem:

```bash
# Formatar um arquivo no local
aipo fmt src/main.aipo

# Verificar se os arquivos estão formatados (modo CI)
aipo fmt --check src/
```

### `aipo package`

Gerenciador de pacotes e dependências hermético do Aipo:

```bash
# Gerar ou atualizar o lockfile determinístico
aipo package lock

# Baixar dependências remotas do GitHub para o cache local
aipo package fetch-github

# Auditar a integridade criptográfica de todas as entradas no cache
aipo package cache verify .aipo/cache

# Limpar entradas de cache obsoletas e não referenciadas
aipo package cache prune .aipo/cache --lock aipo.lock --apply
```
