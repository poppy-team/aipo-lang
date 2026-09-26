# Concorrência & Async

O modelo de concorrência do Aipo é **cooperativo, determinístico e baseado em tempo virtual**. Ele elimina condições de corrida de baixo nível (*data races*) e garante testes 100% reprodutíveis sem sleeps reais na suíte de testes.

---

## Funções Assíncronas (`async fn`)

Funções que realizam operações de I/O, temporização ou comunicação entre processos devem ser anotadas com `async`:

```aipo
async fn buscar_dados(recurso) {
  let timer = task.sleep(50)
  await do
    timer
  end
  return "Dados para " + recurso
}
```

---

## Bloco de Espera Sequencial (`await do ... end`)

Diferente de linguagens em que `await` pode ser inserido arbitrariamente em subexpressões, o Aipo impõe a construção explícita `await do ... end`:

```aipo
async fn executar_fluxo() {
  let tarefa_a = task.spawn(fn() { processar_etapa_1() })
  let tarefa_b = task.spawn(fn() { processar_etapa_2() })

  // Espera sequencial explícita
  await do
    tarefa_a
    tarefa_b
  end

  print("Todas as etapas concluídas!")
}
```

Essa disciplina evita erros de *dangling promises* e tarefas esquecidas (`AIPO_SEM_FORGOTTEN_TASK`).

---

## Combinadores Assíncronos (`task.*`)

A biblioteca padrão fornece combinadores primitivos de alto nível:

- **`task.spawn(fn)`**: Inicia uma nova tarefa concorrente no scheduler cooperativo.
- **`task.sleep(ms)`**: Cria um temporizador baseado em tempo virtual.
- **`task.all(lista)`**: Aguarda até que todas as tarefas da lista tenham sido concluídas com sucesso.
- **`task.race(lista)`**: Retorna assim que a primeira tarefa concluir, cancelando as demais.
- **`task.timeout(tarefa, ms)`**: Cancela a tarefa alvo caso ela exceda o tempo estipulado.
- **`task.cancel(tarefa)`**: Cancela a execução cooperativa de uma tarefa.
- **`task.group()`**: Cria um grupo estruturado de tarefas para ciclo de vida coordenado.

---

## Detecção Transitiva de Ciclos (`AIPO_RT_AWAIT_CYCLE`)

A máquina virtual do Aipo mantém um grafo de espera ativo. Se duas tarefas entrarem em espera mútua (deadlock assíncrono), o runtime detecta o ciclo transitivo imediatamente e dispara a falha determinística `AIPO_RT_AWAIT_CYCLE` com o rastro dos envolvidos.
