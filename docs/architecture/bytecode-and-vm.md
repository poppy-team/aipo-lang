# Bytecode & Máquina Virtual

A execução nativa do Aipo ocorre em uma **máquina virtual de bytecode determinística**, otimizada para baixo consumo de memória e rápido despacho de instruções.

---

## O Formato Binário `.aibc`

O compilador do Aipo pode serializar o bytecode diretamente para um arquivo binário `.aibc`:

- **Cabeçalho Mágico**: 4 bytes `AIBC` seguido pela versão do formato (atualmente `1`).
- **Tabela de Constantes**: Literais inteiros, decimais, strings normalizadas e referências a funções.
- **Tabela de Metadados de Depuração**: Mapeamento de offsets de bytecode para linhas e colunas do código-fonte original.
- **Seção de Código**: Instruções lineares da VM em formato binário compacto.

---

## Arquitetura da Máquina Virtual (`aipo-vm`)

### 1. Modelo de Valores (`Value`)
O tipo `Value` na VM foi otimizado para um tamanho de 48 bytes com variantes empacotadas em `Rc`:
- Primitivos: `Int(i64)`, `Float(f64)`, `Bool(bool)`, `None`
- Heap com contagem de referências: `String(Rc<str>)`, `List(Rc<RefCell<Vec<Value>>>)`, `Dict`, `Set`, `Bytes`
- Objetos de Execução: `Closure`, `BoundMethod`, `TaskHandle`, `HostHandle`

### 2. Otimização do Laço de Despacho
- Instruções decodificadas diretamente via `match` simples, eliminando o custo de `Result` largo de 72 bytes em cada ciclo de CPU.
- Caches monomórficos para resolução de slots de campos em instâncias de estruturas, reduzindo o tempo de acesso a propriedades em até 28%.
- Verificador formal de bytecode que valida limites de slots locais, upvalues e tamanhos de frames antes do início da execução.

### 3. Desassemblador Integrado
A ferramenta `aipo disasm` decompõe o bytecode e relaciona cada mnemônico com a linha exata de código-fonte que o originou, facilitando profiling e auditorias de segurança.
