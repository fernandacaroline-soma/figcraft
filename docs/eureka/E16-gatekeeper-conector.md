# E16: o Gatekeeper como conector do contrato do E15

Branch `eureka/e16-gatekeeper-conector` (26/11). Card: E16 no Backlog do Eureka.

Por decisão da Fernanda (26/11), o alvo do card passou a ser o Gatekeeper do **ds-orchestrator**, que é o usado hoje, no lugar do DS Experts Studio.

## O que mudou

### Um comando só de leitura: `explore.context`

- **O problema:** a ponte do Gatekeeper montava o pedido chamando `explore.choose`. Desde o E15, esse comando é uma decisão da designer e recusa agentes. O Gatekeeper, como agente, não conseguia mais ler o que revisa.
- **A solução:** `explore.context {id}` devolve o mesmo pacote (o lint do DS, o contexto de cada camada e instância, a proposta, a crítica e a marca aplicada) sem escolher nada.
- Ele só responde depois que a designer escolheu a proposta.
- Teste: `a_reviewer_agent_reads_the_chosen_proposals_context_without_choosing`.

### A ponte (`gatekeeper_bridge.py`) com identidade de agente

- **Identidade:** ela fala com o app como o agente **"DS Gatekeeper"**, com a chave de sessão.
- **Leitura:** usa `explore.context` e nunca escolhe a proposta.
- **Parecer:** posta com `explore.review`, marcado como desse agente.
- **Limite:** como agente, o Gatekeeper não consegue escolher, revisar a crítica nem aprovar o layout. Essas são decisões da designer.

## Demonstração no app real (proposta G, base do template, NV)

1. **A pessoa:** escolheu a G e revisou a crítica. Foi um teste; a escolha real é da designer.
2. **O Gatekeeper:** montou o pedido lendo o contexto como agente (`gk-request.json`).
3. **O parecer:** o agente `ds-orchestrator:ds-gatekeeper` revisou, só lendo e sem registro no Notion. Resultado: **com ressalvas, 6 itens**, cada um com a fonte (`demo-e16/E16-parecer-G.json`):
   - dois botões primários (o "Eu quero" da galeria e o "Comprar agora · M");
   - o tamanho "M" no botão e no Snackbar, com a grade numérica e nenhum tamanho selecionado;
   - estados que faltam: cliente sem tamanho conhecido ou sem login, e sem estoque;
   - o destino do "Comprar agora", com os estados de erro e de carregamento;
   - as peças do template soltas: para handoff, a tela tem de partir de uma instância do template;
   - o contraste de 3,6:1 que vem do componente do DS (corrigir na origem, não na instância).
4. **Gravação:** a ponte postou o parecer como "DS Gatekeeper". Ele ficou guardado na proposta, com o revisor identificado.
5. **Limite:** o Gatekeeper tentou aprovar o layout e o app recusou ("this is the designer's decision"). A proposta continua sem aprovação.

## O que falta no E16

- **Checkpoint humano:** revisar o fluxo.
- **Registro no Notion:** registrar o parecer no Registro de alterações do DS continua desligado no modo prévia. Ligar depende de aprovação.
- **Disparo:** quem pede o parecer ainda é um operador: o Claude chama o agente entre a exportação e a postagem. O app não dispara o Gatekeeper sozinho.
- **Itens do parecer:** os que são decisão da liderança de DS estão em `docs/eureka/PENDENCIAS-DECISAO.md`.
