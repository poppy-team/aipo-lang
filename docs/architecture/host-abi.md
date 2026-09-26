# Host ABI & Sandboxing

A **Host ABI (`aipo-host`)** define a interface de isolamento e segurança entre os scripts em Aipo e o ambiente que hospeda o runtime (seja um jogo, uma aplicação de backend ou um agente inteligente).

---

## O Modelo de Permissões Deny-by-Default

Por padrão, um script em Aipo roda em um ambiente completamente isolado:
- Não pode ler variáveis de ambiente (`env`).
- Não pode ler ou gravar no sistema de arquivos (`fs`).
- Não pode consultar o relógio do sistema operacional (`clock`).
- Não pode abrir conexões de rede.

Para que um recurso seja acessível, a aplicação anfitriã em Rust deve registrar uma permissão explícita na árvore de capacidades (`CapabilitySet`):

```rust
// Exemplo no anfitrião em Rust: concedendo apenas relógio e leitura em diretório restrito
let mut caps = CapabilitySet::new();
caps.grant("clock");
caps.grant("fs.read");
```

Se o script tentar acessar uma função sem a permissão correspondente, o runtime dispara a falha determinística `AIPO_RT_CAPABILITY_DENIED`.

---

## Handles Geracionais contra *Use-After-Free*

Para objetos complexos do host (como janelas de UI, entidades de jogo ou conexões com banco de dados), o Aipo utiliza a tabela de handles geracionais (`HandleTable`):

- Cada handle contém um **índice de slot** e um **número de geração**.
- Quando o objeto do host é destruído ou reciclado, a geração do slot é incrementada.
- Qualquer tentativa posterior do script de acessar o handle antigo é rejeitada com o erro `AIPO_RT_STALE_HANDLE`, eliminando ponteiros soltos e corrupção de memória.
