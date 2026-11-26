# Frente U: rascunho da memória visual de um layout aprovado

Branch `eureka/u-memoria-visual` (26/11). Faz parte do E17 (G3) e fecha a esteira da skill
gerador-propostas no Eureka.

## O problema

A Fase E da skill (`references/memoria-visual.md`) diz que cada layout **aprovado** ensina a
identidade visual da marca. O agente monta uma entrada (componentes e tokens, padrões de layout,
imagem, texto, interações, o que a designer ajustou, o que evitar) e só grava no Notion com o ok
explícito da designer. No Eureka, a esteira parava no parecer do Gatekeeper.

## O que muda

- **`explore.approve`.** A designer aprova o layout refinado da proposta escolhida. A aprovação só
  é aceita depois de duas etapas, na ordem da skill: a crítica revisada e o parecer do Gatekeeper.
  Ela fica guardada na proposta. Escolher outra proposta apaga a aprovação, porque ela era do
  outro layout.
- **Botão "Aprovar o layout" no cartão.** Aparece depois do parecer. Depois do clique, o cartão
  mostra "Você aprovou este layout."
- **`explore.memory`.** Só leitura. Monta a entrada **no modelo da skill**, em Markdown e em campos
  separados, com `written: false`:
  - **Componentes e tokens do DS:** o que a proposta usou.
  - **Padrões de layout:** os blocos de cima para baixo e os tokens de espaçamento da própria tela
    (sem os de dentro dos componentes).
  - **Imagens.**
  - **Chamadas para ação:** os textos dos botões, sem nomes de ícone nem rótulos de tamanho.
  - **Estados mostrados:** pela mesma regra da crítica, sem contar tipos como o Status de uma
    etiqueta.
  - **O que a designer ajustou.** Sai do **registro de origem**: só o que foi mudado à mão, não o
    que o agente desenhou. Os ajustes ficam agrupados pela instância a que pertencem. Para a skill,
    esse é o sinal mais forte da memória.
  - **O que evitar:** as propostas não escolhidas, com o motivo a registrar pela designer.
  - **Status:** "observação".
  - **Validação:** "sem teste ainda".

  Os campos que o documento não tem (link do Registro e link do layout) entram como espaço a
  preencher. A gravação no Notion continua com a skill, depois do ok da designer.

## Resultado na G (`demo-g3u/U-memoria-G.md`)

A rodada usou a tela base do template oficial (Frente T). O fluxo completo na G foi: crítica
revisada, parecer, um ajuste à mão no texto do Snackbar e aprovação. Tudo isso foi teste: a
aprovação real é da designer.

A entrada lista:
- os componentes do template;
- o Snackbar e o Action Button;
- os tokens da tela;
- as chamadas "Comprar agora · M" e "Adicionar à sacola";
- os estados mostrados;
- **"O que a designer ajustou: Snackbar (props, characters)"**: exatamente o ajuste feito, e nada
  do que o agente desenhou.

## Teste

`an_approved_layout_drafts_its_visual_memory_entry_with_the_designers_adjustments` cobre:

- **ordem:** a aprovação é recusada sem a crítica revisada e sem o parecer;
- **data:** formato AAAA-MM-DD;
- **entrada:** começa com o título no modelo da skill;
- **ajustes:** só o da designer;
- **o que evitar:** a proposta não escolhida;
- **troca de escolha:** a aprovação anterior é apagada.

## Limites

- **Campos que só a designer preenche:** "Imagem" (tipo e enquadramento), o motivo de cada proposta
  não escolhida e a validação com clientes.
- **Data:** vem de quem chama. O Eureka não usa o relógio.
- **Gravação:** a entrada não é gravada em lugar nenhum. A página "Memória visual — <Marca>" no
  Notion continua com a skill, com ok explícito.
