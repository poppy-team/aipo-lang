# Prumo CLI & Metodologia LPC

O Aipo utiliza o **Prumo v0.6** como framework de governança contínua e a metodologia **Lean Progressive Context (LPC)** para coordenação entre desenvolvedores humanos e agentes de inteligência artificial.

---

## O Princípio do Lean Progressive Context (LPC)

1. **Menor Contexto Suficiente**: Cada tarefa inicia com a carga mínima de arquivos necessários, evitando poluição de contexto e alucinações.
2. **Ponteiro sobre Carga Útil (*Pointer over Payload*)**: Utilização de mapas de autoridade (`AUTHORITY_MAP.json`), contratos estáveis e referências formais em vez de ler o repositório inteiro.
3. **Expansão Progressiva Delimitada**: O contexto é expandido apenas quando a evidência disponível for insuficiente para garantir o critério de aceitação.
4. **Nunca Enfraquecer Critérios Silenciosamente**: Se uma meta técnica encontrar um obstáculo não previsto, a questão torna-se um ADP explícito e nunca uma concessão silenciosa.

---

## O Prumo como Ferramenta de Ciclo de Vida

O utilitário `prumo` orquestra:
- **Portões de Qualidade**: Verificação determinística de cobertura documental (`prumo docs audit`, `prumo docs verify --strict`).
- **Rastreabilidade de Evidências**: Mapeamento de decisões, artefatos gerados e histórico de execução em `.prumo/history/`.
- **Prevenção de Derivação Documental**: Sincronização obrigatória entre especificações canônicas, testes automatizados e código de produção em cada alteração (*Documentation Delta*).
