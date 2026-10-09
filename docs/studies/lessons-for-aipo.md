# Lições das referências e plano de aplicação na Aipo

**Base:** `488905ffb0b83ebbff458939542350d7280ad79b`. **Rodada:** reconstrução de 2026-10-09, sem executar testes por solicitação do usuário.

## O que virou implementação

| Lição | Referência | Aplicação entregue | Limite |
| --- | --- | --- | --- |
| Frames têm estado e lifetime explícitos | Wren, Lua, LuaJIT | Handlers isolados por ativação; caller restaurado em retorno/fault | Ainda salva 256 registradores por chamada |
| Campos de bytecode têm papéis específicos | LuaJIT | Checagem de B por opcode, jumps/pool/calls e iteração com C completo | Falta verificador integral Reg |
| Uma semântica não deve ser duplicada em emissores | Frontend compartilhado Aipo | Aritmética/igualdade Reg delegadas a Value; Dict corrigido VM/Reg/JS | Paridade geral continua incompleta |
| Contagens e buffers exigem limites | Luau, wasm3 | Bytes rejeita categorias inválidas; Reg tem budget; Wasm expõe fuel | Não cobre memória/output/tempo de host |
| Erros de engine precisam chegar ao embedding | Runtimes Wasm e contrato Rust | MissingExport e erros do writer propagados | Binding de string mantém silêncio em ponteiros inválidos |
| Layouts e ganhos dependem de alvo/flags | QuickJS, Janet, LuaJIT | Probe de layout e harness de medição reproduzível | Nenhum resultado quantitativo novo |
| Features do núcleo diferem de integração | wasmi, WAMR | Documentação corrige confusão no_std/WASI e mantém runner opcional | Nenhum runtime alternativo integrado |

As correções semânticas vêm do canon da Aipo. Referências externas ajudam a entender mecanismos; não têm autoridade para mudar a linguagem.

## Propostas que continuam propostas

**Janelas de registradores:** guardar base, limite e return PC por ativação, ampliar armazenamento por slots e fechar capturas antes de invalidar frames. Exige semântica de unwind e bench de chamadas, não apenas substituir um array por um Vec.

**Shapes/slots/atoms:** separar identidade de tipo, nomes internados e posição de fields; manter fallback para layouts diferentes e medir acertos/falhas do cache. Não transformar structs em objetos com a semântica de JS.

**Closures e async Reg:** implementar células compartilhadas, self-capture, await e scheduler conforme o runtime existente. Rejeitar programas sem suporte permanece melhor que omitir instruções.

**Versionamento estrutural:** cada operação de adição/remoção incrementa uma versão; IterGuard registra versão e identidade. Isso fecha a lacuna de alterações que preservam comprimento, mas exige alterar todos os mutadores e a política de callbacks.

**Host/sandbox:** orçamento de bytecode e fuel não limitam callbacks bloqueantes, capacidade de filesystem/processos, memória nem saída. Reutilizar o contrato AHS e capabilities da Aipo; não presumir que copiar globais da Stack VM instala seus serviços no RegVM.

**Alternativas Wasm:** comparar wasmi, WAMR e outros candidatos com os mesmos módulos, imports e checksums. O núcleo wasmi no_std e seu adaptador WASI/std são caminhos distintos. Seleção exige medições e decisão documentada.

**Perfis shell/nano:** auditar grafo por features e medir startup/tamanho reais; o perfil Cargo atual não entrega no_std. Manter os perfis no mesmo workspace e preservar uma semântica.

## Ordem de continuidade

1. Executar, em rodada autorizada futura, conformance e os gates desta reconstrução.
2. Verificador integral Reg, versionamento de coleções e políticas de host/output.
3. Paridade de métodos, closures, async, hooks e contratos estruturais.
4. Janelas/slots e caches com benchmark pareado, sem transportar resultados de upstream.
5. Perfil nano/shell e spike de engine alternativa com decisão de licença e portabilidade.

O [guia de uso e reimplementação](runtime-hardening-guide.md) contém APIs, mudanças de compatibilidade e passos de cada correção. O [guia master](llm-study-guide.md) mantém a metodologia e os critérios de aceite que ainda não foram certificados.
