# Backend JavaScript (`aipo-js`)

O backend JavaScript do Aipo permite transpilar código-fonte Aipo para **JavaScript moderno (ES2022)** com garantia de paridade comportamental total.

---

## O Desafio da Paridade

Muitos transpiladores para JavaScript caem na armadilha de usar os tipos primitivos do JavaScript de forma frouxa, gerando inconsistências semânticas sutis (como `0 == ""` ou `"5" + 2 == "52"`).

O `aipo-js` adota uma estratégia rigorosa:
1. **Runtime Shim Versionado**: Um pequeno pacote auxiliar em JavaScript puro que expõe classes e funções para manter a semântica de tipos fortes, arrays, mapas com ordem e manipulação de bytes.
2. **Emissão Limpa**: Transpila estruturas de controle para equivalentes em JS (funções, classes, `try/catch` para `attempt`), preservando a performance do motor V8 ou JavaScriptCore.
3. **Mapeamento de Linhas com Source Maps**: Geração automática de Source Maps V3, permitindo depuração diretamente nas ferramentas do navegador ou no Node.js usando o arquivo `.aipo` original.

---

## Verificação por Testes Diferenciais

Para assegurar que o compilador JavaScript nunca divirja da máquina virtual em Rust, o projeto mantém uma suíte de testes diferenciais contínuos:
- Todo programa em `docs/conformance/programs/` é executado na VM em Rust e no Node.js.
- Se houver discrepância em um único caractere da saída padrão ou em um código de diagnóstico, o build do projeto é bloqueado no CI.
