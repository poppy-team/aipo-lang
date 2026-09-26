# Wave 0 — MVP da Linguagem em 11 Slices

A **Wave 0** estabeleceu os alicerces fundamentais da linguagem Aipo através de 11 fatias verticais (*vertical slices*) rigorosamente integradas e testadas.

---

## Estrutura dos 11 Slices

### S1: Workspace, Source & Diagnostics (`aipo-source`, `aipo-diagnostics`)
- Configuração do workspace em Rust com edição 2024.
- Representação canônica de arquivos de código-fonte (`Source`) com cálculo seguro de limites de caracteres UTF-8 (`is_char_boundary`).
- Estrutura de diagnósticos com códigos de erro estáveis, spans de localização precisos e mensagens amigáveis.

### S2: Lexer Core (`aipo-lexer`)
- Tokenizador de alta performance sem alocação desnecessária para palavras-chave e identificadores.
- Rejeição rigorosa de literais numéricos malformados (`1.e5`, `1._5`) emitindo `AIPO_LEX_INVALID_NUMBER`.

### S3: Parser Core & AST (`aipo-syntax`, `aipo-ast`)
- Parser descendente recursivo com recuperação automática de erros (`synchronize`).
- Árvore sintática abstrata tipada cobrindo expressões de valor, estruturas, funções e declarações em nível de topo.

### S4: HIR Lowering (`aipo-hir`)
- Conversão da AST em High-level Intermediate Representation (HIR).
- Desaçucaramento de sintaxes idiomáticas e unificação de spans para hooks estruturais.

### S5: Semantic Analysis (`aipo-sema`)
- Tabela de símbolos léxica com suporte a escopos aninhados e sombras controladas.
- Validação estática de declarações `struct`, detecção de campos duplicados e verificação inicial de tipos.

### S6: Core IR & Bytecode (`aipo-ir`, `aipo-bytecode`)
- Representação intermediária linearizada focada em controle de fluxo explícito.
- Emissor de bytecode determinístico gerando instruções compactas para a máquina virtual.

### S7: VM Core (`aipo-vm`)
- Interpretador baseado em pilha e registradores lógicos de execução contínua.
- Pilha de frames de chamada (`CallFrame`) com isolamento seguro de locais e temporários.

### S8: Data & Errors (`aipo-vm`)
- Modelo de valores `Value` com representação otimizada.
- Sistema de falhas estruturadas (`VmFault`) mapeadas para diagnósticos amigáveis ao usuário.

### S9: Runtime & Stdlib Mínima (`aipo-runtime`, `aipo-stdlib`)
- Registro de funções nativas do host.
- Módulos matemáticos básicos, manipulação de texto e saída padrão (`print`).

### S10: CLI & Formatter (`aipo-cli`, `aipo-formatter`)
- Subcomandos de linha de comando `aipo run`, `aipo check` e `aipo disasm`.
- Formatador automático de código baseado em regras canônicas de indentação e espaçamento.

### S11: Suíte de Conformance Inicial
- Conjunto inicial de testes end-to-end garantindo que programas canônicos compilam e executam com a saída exata esperada.
