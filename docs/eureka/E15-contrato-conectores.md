# E15: contrato dos conectores de agentes e permissões

Branch `eureka/e15-conectores` (26/11). Card: E15 no Backlog do Eureka (Notion).

O card pede um contrato para os agentes que trabalham no Eureka cobrindo:
- entradas e saídas, contexto e versionamento;
- limites, progresso e logs;
- permissões e aprovação humana;
- agentes desacoplados do editor.

Critérios de aceite: mock conectado, erro e cancelamento, aprovação humana e isolamento demonstrados.

Agentes de hoje:
- a skill gerador-propostas, da Naty;
- o DS Gatekeeper;
- os scripts do Eureka;
- o MCP (`figcraft-cli mcp --attach`);
- o outro computador.

## O contrato

### 1. Transporte e isolamento

- O agente fala com o app pela conexão local de controle (`figcraft --control PORTA`).
- Essa conexão só aceita conexões da própria máquina (`127.0.0.1`).
- **Chave de sessão (novo):**
  - Toda conexão começa com `{"method": "auth", "params": {"token": …}}`.
  - Ao abrir, o app gera a chave, grava num arquivo que só o usuário lê (`figcraft-control-<PORTA>.token` na pasta temporária do usuário) e mostra o caminho.
  - Quem inicia o app pode fixar a chave com `FIGCRAFT_CONTROL_TOKEN`, como os scripts e o CI fazem.
  - Sem a chave, ou com a chave errada, nenhum comando roda e a conexão termina, sem segunda tentativa.
  - A comparação da chave não vaza onde ela difere.
  - Se o app não tiver como gerar a chave, a conexão nem abre.
- **Pastas:** `--automation-read-root` e `--automation-write-root` limitam onde o app lê e grava. Isso já existia e continua opcional.
- **Plugins:** rodam em sandbox (wasmi), sem acesso a arquivos. Também já existia.

### 2. Versão

- A resposta do `auth` traz `protocol` (hoje 1) e a versão do app.
- Se o conector precisar de uma versão mais nova, ele para ali, antes de mandar qualquer comando.
- Os comandos têm ids estáveis (`explore.board`, `brand.apply`…). Cada um descreve no próprio texto de ajuda os parâmetros que aceita e o que devolve, e o app recusa parâmetros que não conhece.

### 3. Entradas e saídas

- **Pedido:** `{"id", "method", "params"}`, uma linha JSON por vez.
- **Resposta:** `{"id", "ok": true, "result"}` ou `{"id", "ok": false, "error"}`.
- **Agente:** os comandos vão por `engine.execute {command, params, agent}`. O campo `agent` diz qual agente está agindo.
- **Desacoplamento:** o agente nunca mexe no editor diretamente. Ele descreve o que quer, por exemplo a tela pela descrição de componentes, variantes e tokens, e o app confere tudo antes de desenhar.

### 4. Contexto

O agente lê, sem alterar nada:
- `explore.catalog`: componentes e tokens do DS disponíveis;
- `explore.choose`: o contexto da proposta escolhida, com o lint do DS;
- `brand.profile`: o perfil da marca;
- `export.codeMap`: o mapa de código;
- `ui.inspect`: o estado da tela.

### 5. Limites

Os limites que já existiam valem para todo agente:
- mensagens de até 32 MB;
- até 8 conexões e 8 pedidos na fila;
- 60 s por pedido, que o pedido pode estender até 10 min com `timeoutMs`;
- arquivos de no máximo 4 MB a 16 GB, conforme o tipo;
- limites próprios em cada comando do Eureka (propostas por quadro, chamadas por arquivo e outros).

### 6. Progresso e logs

- `document.checkpoint` e `document.changesSince`: o que mudou desde um ponto.
- `events.poll`: os eventos do app.
- Origem por camada: cada camada guarda se foi feita pela IA ou por uma pessoa, com o nome do agente. Isso vem da G2.
- O agente pode deixar comentários assinados.

### 7. Erro e cancelamento

- **Erro:** um comando que falha não muda nada. O documento volta ao estado anterior, porque cada comando é uma única etapa de desfazer.
- **Cancelamento:** um pedido que o app ainda não começou é cancelado quando vence o tempo e nunca roda. Um pedido já iniciado termina, e o erro avisa que ele pode ter sido aplicado. Os dois casos têm testes no servidor de controle (`a_request_cancelled_before_it_started_never_runs`, `a_started_request_times_out_as_still_running`).
- **Lote:** `command.batch` com `dryRun` mostra o que mudaria sem mudar nada.

### 8. Aprovação humana (novo)

- O agente prepara e recomenda. Quem decide é a designer.
- As decisões da pessoa recusam qualquer comando que venha com `agent`, inclusive dentro de um `command.batch`:
  - `explore.choose` (escolher a proposta);
  - `explore.critiqueStatus` e `explore.critiqueReviewed` (marcar e fechar a revisão da crítica);
  - `explore.approve` (aprovar o layout);
  - `proposal.apply` (aplicar uma proposta).
- Os botões do cartão rodam essas decisões em nome da pessoa.
- **Limite honesto:** essa regra protege contra agentes que se identificam, que é o caso de todos os nossos. Um programa que se conecta sem dizer quem é seria tratado como pessoa. Por isso a chave de sessão importa: só quem tem a chave entra.

### 9. O que um agente nunca faz

Estas regras continuam valendo e são de governança, não de código:
- alterar tokens, componentes ou regras do DS;
- excluir no Figma;
- gravar no Notion sem o ok explícito;
- decidir pela designer.

## Demonstração: conector simulado

`docs/eureka/conector_mock.py` conversa com o app real como "Mock agent". Resultado (`E15-conector.txt`):

1. Sem chave: recusado, conexão encerrada. Com chave errada: "wrong token", conexão encerrada.
2. Com a chave certa: FigCraft 0.18.1, protocolo 1.
3. Contexto: lê o catálogo.
4. Trabalho: monta um quadro com 2 propostas, como agente.
5. Erro: um quadro inválido é recusado, e 0 camadas mudam.
6. Progresso: 3 camadas criadas desde o início do trabalho.
7. Aprovação humana: o agente tenta escolher a proposta e o app recusa ("this is the designer's decision"). A pessoa, sem `agent`, consegue.

Também conferi:
- o arquivo da chave fica com permissão `-rw-------`, só do usuário;
- a rodada completa da NV (`rodada_n.py`) roda com a chave.

## O que muda para quem conecta

- **Scripts do Eureka:** os 19 que conectam ao app passaram a enviar a chave (`docs/eureka/eureka_control.py`).
- **MCP (`figcraft-cli mcp --attach`):** envia a chave sozinho. Ela vem de `FIGCRAFT_CONTROL_TOKEN` ou do arquivo da porta.
- **Outro computador:** os scripts dele precisam da mesma linha. Aviso na fila.
- **Qualquer outro cliente:** acrescentar o `auth` no começo da conexão (`docs/control-protocol.md`).

## Validação

- **Servidor de controle:**
  - 27 testes, com 4 novos: sem chave nada roda e a conexão fecha; chave errada é recusada sem segunda tentativa; chave certa abre a conexão; o arquivo é privado e substitui um anterior.
  - A comparação da chave tem teste próprio.
  - O teste "sem chave" falha quando a verificação é removida.
- **MCP:** a chave sai do arquivo e vai antes do primeiro pedido.
- **Motor:** `an_agent_cannot_make_the_designers_decisions`, com as 5 decisões, o lote e a pessoa.
- **Docs:** `cargo xtask docs` passa.

## Para decidir

- **Checkpoint humano (card E15):** revisar o contrato.
- **Pastas restritas por padrão:** hoje elas continuam opcionais. Restringir por padrão com `--control` pede uma lista de pastas permitidas, que é decisão sua.
- **Pedido de cancelamento explícito:** hoje o cancelamento é por tempo. Um método `cancel {id}` fica para quando um agente precisar dele.
