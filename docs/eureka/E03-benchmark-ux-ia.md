# E03 — Benchmark UX de IA integrada a canvas

Data de acesso a todas as fontes: 13/11/2026. Pesquisador: Product Researcher (Claude). A decisão final é da dona do produto.

Legenda de confiança: **[V]** Verificado (oficial) · **[S]** Secundário (fórum, blog, resumo de busca) · **[NE]** Não encontrado. Os números entre colchetes remetem à lista de Fontes.

## 1. Resumo

1. Quase todas as ferramentas seguem o mesmo laço: selecionar no preview, pedir, esperar a IA, aceitar ou reverter por versão.
2. Só v0, Bolt, Figma Make e Cursor (em lote) mostram edições **pendentes** antes de aplicar. Subframe mostra variações com Keep/Discard.
3. Nenhuma fonte oficial descreve **origem por camada IA × humano**, nem **desfazer só a IA** preservando edições manuais posteriores.
4. O desfazer é por versão do projeto inteiro (Lovable, Figma Make, Bolt). Lovable diz que não há reversão parcial.
5. Acessibilidade da experiência de IA só aparece de forma concreta no Claude Design (leitor de tela, teclado, contraste). Nas demais, é escassa ou ausente.

## 2. Matriz

Perguntas: P1 chamar a IA · P2 selecionar e pedir · P3 sugestões sem interromper · P4 diff e aprovação · P5 execução, erro, cancelamento, histórico · P6 edições manuais preservadas · P7 dependência de chat · Ac acessibilidade.

### 2.1 Claude Design (Anthropic Labs)
- **P1** [V] Chat à esquerda, canvas à direita [2][3].
- **P2** [V] Comentário inline no elemento, edição direta no canvas e chat. Comentário serve para ajuste de componente; chat para mudança estrutural [3].
- **P3** [S] Controles tipo slider ("Tweaks") aparecem só em guias de terceiros. O anúncio oficial cita "custom sliders" [2]; o artigo de suporte não os descreve [3].
- **P4** [NE] Não há aprovação nem diff documentados [2][3].
- **P5** [V] Sem histórico de versões ainda. Cmd+Z / Cmd+Shift+Z funcionam. Dá para pedir ao Claude que salve o projeto antes de explorar outra direção. Problema conhecido: comentário às vezes não é lido; colar no chat [3].
- **P6** [NE] O suporte não explica como edições manuais interagem com mudanças posteriores do Claude [3].
- **P7** [V] Três canais (chat, comentário, edição direta). Chat não é o único [3].
- **Ac** [V] Leitor de tela anuncia seleção e posição; seletor de cor informa contraste; F6 troca de região; atalhos para mover camada. Criar texto/forma exige mouse, ou pedir ao Claude [3].

### 2.2 Figma Make
- **P1** [V] Prompt no chat; fila de prompts reordenável; modo plano para builds complexos [4].
- **P2** [V] Ferramenta **Edit** (antes "Point and Edit") seleciona elemento e abre painel de propriedades; anotação ("Annotate for agent") vincula instrução a uma área. Multi-seleção por arrastar ou Shift [4][5][6]. Texto vindo de dados não é editável pela ferramenta [S][7].
- **P3** [V] Edições do painel e anotações ficam **em stage** no campo de prompt antes de aplicar [5].
- **P4** [V] É possível revisar cada edição em stage e descartá-la. Ao aplicar, vira nova versão e consome créditos; stage não consome [5]. Diff visual: [NE].
- **P5** [V] Versão automática a cada edição sua ou da IA, com nome gerado; preview de versão antiga sem alterar o trabalho atual; restaurar não apaga versões posteriores [8]. Cancelamento e erro: [NE]. Undo: [NE] [5].
- **P6** [NE] Não documentado.
- **P7** [V] Médio. Painel de propriedades e anotações reduzem o uso de chat [5].
- **Ac** [V parcial] Só ditado por voz e orientação de pôr acessibilidade no prompt [4]. Atalhos: [NE].
- **Correção:** "checkpoint por turno" não se confirma. A fonte oficial fala em versão a cada edição, sem amarrar a turno [8]. "Perder a seleção ao enviar": [NE] como regra; fóruns relatam outros problemas (troca de página, botão ausente, prompt desabilitado sem créditos) [S][7].

### 2.3 v0 (Design Mode)
- **P1** [V] Botão Design Mode na barra do preview; só no chat mais recente, não em mobile [9].
- **P2** [V] Hover destaca, clique seleciona. Painel visual (tipografia, cor, margem, borda, sombra) ou campo de instruções; anexa screenshot do elemento. Os dois podem ser combinados [9].
- **P3** [V] Edições do painel aparecem ao vivo no preview e ficam pendentes [9].
- **P4** [V] Toggle antes/depois, undo/redo, Reset por elemento, botão **Apply**. Cada Apply gera versão com diff revisável [9]. Aceitar parcial dentro do lote: [NE].
- **P5** [V] Aviso ao sair com edições pendentes. Reverter como qualquer versão [9]. Cancelamento da geração: [NE].
- **P6** [S] Fórum relata que a IA altera elementos vizinhos mesmo com pedido restrito [10]. Docs oficiais: [NE]. A API doc diz que não é patch determinístico de DOM; um modelo traduz as edições em código [V][11].
- **P7** [V] Baixo para ajustes visuais; chat para mudança estrutural [9].
- **Ac** [V parcial] Só Cmd/Ctrl+I (alternar Inspect) e Esc [9].

### 2.4 Lovable
- **P1** [V] Barra de ferramentas do preview [12][13].
- **P2** [V] **Select elements** (tecla S); Cmd/Ctrl+clique soma elementos; cada um vira referência no campo de chat. **Draw annotation** (tecla D). Comentários fixados [12][13].
- **P3** [V] Pedidos de seleção e desenho entram "no próximo ponto natural de parada"; texto inline e comentários rodam depois da tarefa atual [12].
- **P4** [NE] Sem diff nem aprovação por item documentados.
- **P5** [V] Toda mudança cria versão. Prévia e reverter; reversão restaura o projeto inteiro, **sem reversão parcial**; para desfazer só um pedaço, pedir no chat. Favoritar versões estáveis. Dados de banco não são revertidos [14].
- **P6** [NE] A página não diz se edição manual de código cria versão [14].
- **P7** [V] Alto: seleção alimenta o chat. Edição de texto inline é exceção [12][13].
- **Ac** [NE] Fora das teclas S e D.

### 2.5 Bolt
- **P1** [V] Ferramenta Select no campo de chat [15].
- **P2** [V] Clique no elemento; texto por duplo clique; barra para cor, fonte, alinhamento, opacidade [15].
- **P3** [V] Cada elemento editado ganha marcador no preview; edições acumulam num lote acima do chat; editar no preview é grátis [15].
- **P4** [V] Revisar o lote e apagar edições uma a uma ou todas; **Save changes** aplica tudo ao código [15].
- **P5** [V] Esc ou clicar fora reverte edição ainda não adicionada. Depois de salvo, não dá para desfazer item individual; usar histórico de versões do projeto [15].
- **P6** [NE] Não documentado.
- **P7** [V] Baixo no caminho visual. No caminho por prompt, a mudança aparece só quando o Bolt termina [15].
- **Ac** [NE] Esc apenas.
- **Confirmado:** "sem preview no caminho por prompt" [V][15].

### 2.6 Subframe
- **P1** [V] Botão Agent ou Cmd+/; chat à direita. Em um componente, clique direito ou "/" > "Ask agent to edit..." [16].
- **P2** [V] Selecionar elemento e usar "/" ou clique direito para **quick edit**; também @ para referenciar páginas e componentes [16][17].
- **P3** [V] Alternativas aparecem lado a lado no canvas; mudanças simples vão direto [16][17].
- **P4** [V] Clicar na variação mostra prévia; **Keep** aplica, **Discard** descarta; cartão "Original" para comparar; é possível arrastar elementos de uma variação para o canvas e misturar [16][17][18]. Diff por item: [NE].
- **P5** [V] Esc para a resposta; mensagens digitadas durante a execução entram em fila; conversas persistem; Cmd+Z desfaz edição no editor visual, mas não mudanças via MCP/API (restaurar pelo histórico de versões) [16].
- **P6** [NE] Não documentado.
- **P7** [V] Médio. Quick edit na seleção, chat para páginas inteiras [16][17].
- **Ac** [NE] Atalhos apenas.
- **Correção:** a contagem é **1–4** na página principal [16][18] e **até 3** em quick edit [17]; edições precisas retornam 1 resultado.

### 2.7 Google Stitch
- **P1** [V] Chat, imagem, seletores de tema [19].
- **P2** [NE] Seleção de elemento na tela não documentada.
- **P3** [V parcial] Gera várias variantes para comparar layout, componente e estilo [19]. "Variações por eixo": [NE]. Tutorial de terceiros fala em 2–5 variações e "Creative Range" [S][20].
- **P4** [NE] Aprovação e diff.
- **P5** [NE] Histórico e erro. Fórum: erro "Could not copy the design" [S][21].
- **P6** [NE].
- **P7** [V parcial] Alto: refinamento por chat, imagem e tema [19].
- **Ac** [NE].
- **Figma:** o post oficial confirma "paste to Figma" [19]. "Exportar perde componentes": [NE]. Fóruns indicam limites por modo (Standard/Rapid) e por modelo, com relatos de layout quebrado [S][21][22].

### 2.8 Cursor
- **P1** [V] Navegador no Agents Window; botão Annotate, Cmd/Ctrl+Shift+D ou paleta de comandos [23].
- **P2** [V] Clique abre nota no elemento; setas cima/baixo sobem ou descem na árvore; arrastar marca área; duplo clique edita texto; Shift+clique faz multisseleção; inspetor de CSS para um elemento. A nota envia seletor, estilos, nome do componente (React, Vue, Svelte) e screenshot recortada [23].
- **P3** [V] Com "Batch edits" ligado, as notas viram pinos numerados e enviam juntas [23]. Barra lateral de design (blog de dez/2025) tem sliders, paletas e tokens de cor/DS do time [24].
- **P4** [V parcial] Lista de notas pendentes antes de enviar, "Clear All" [23]. Diff ou aprovação das edições de código do agente: [NE] nesta página.
- **P5** [V] Esc cancela o passo; hot reload mostra o resultado; Cmd+Z desfaz só a marcação, **não** as edições do agente [23]. Histórico: [NE].
- **P6** [NE].
- **P7** [V] Médio: nota na seleção e chat; atalhos para mover nota ao chat [23].
- **Ac** [NE].
- **Correção:** "editor visual com tokens do DS" vale para o blog de 2025 [24]. Hoje o recurso é o Design Mode com Annotate [23]; a data do Cursor 3 e de canvases vem de terceiros [S][25].

### 2.9 Penpot
- **P1** [V] Cliente MCP externo (Cursor, Claude Code). Em Penpot: File > MCP Server > Connect [26].
- **P2** [NE] Seleção não documentada; o MCP age na **página focada** [26].
- **P3** [NE].
- **P4** [NE] A página não descreve revisão. Recomenda pedir ao agente que descreva as mudanças antes e fazer passos pequenos e reversíveis [26]. O blog orienta salvar versão ou duplicar o arquivo antes [27].
- **P5** [V] Só uma aba controla o MCP; a aba precisa ficar ativa [26]. Histórico: via versão do Penpot [27].
- **P6** [NE].
- **P7** [V] Total: o chat fica no cliente externo [26].
- **Ac** [NE].
- **Confirmado:** servidor MCP + plugin [26].

### 2.10 Builder.io (Visual Editor AI / Fusion)
- **P1** [V] Aba Generate; AI Chat com Visual Copilot [28].
- **P2** [V parcial] Copiar referência da parte a editar; docs de Fusion citam ícone Select Element [S][29]. A página fetchada não descreve clique direto [28].
- **P3** [NE].
- **P4** [V] Aplicar ou descartar as mudanças ao fim [28]. Sugestões geradas: Accept ou Reject [S][29].
- **P5/P6** [NE].
- **P7** [V] Médio [28].
- **Ac** [NE].

### 2.11 Onlook
- Todas as células: [S]. README e comparações dizem editor visual com IA sobre Next.js e Tailwind, com sincronização com o código e clique direito para abrir o código [30]. Undo e revisão: [NE].

### 2.12 Framer
- **P2** [S] Botão Select sob o campo do agente; o agente só trabalha com o elemento selecionado [31][32].
- **P5** [S] Cada mensagem teria ponto de retorno próprio e mudanças ficariam em branch; guia comunitário, **não oficial** [31]. A página oficial do glossário sobre branch não traz conteúdo [33].
- **Demais** [NE]. Observação [S]: agentes externos não veem a seleção; plugin Context Picker contorna [32].

## 3. Fontes

1. Busca web (padrão), vários resultados, 13/11/2026 — sinalizada nas notas abaixo.
2. Anthropic, "Introducing Claude Design by Anthropic Labs" — https://www.anthropic.com/news/claude-design-anthropic-labs (13/11/2026)
3. Suporte Claude, "Get started with Claude Design" — https://support.claude.com/en/articles/14604416-get-started-with-claude-design (13/11/2026)
4. Figma Help, "Create a Figma Make file" — https://help.figma.com/hc/en-us/articles/31304485164695 (13/11/2026)
5. Figma Blog, "Properties panel and annotations now in Figma Make" — https://figma.com/blog/properties-panel-and-annotations-now-in-figma-make (13/11/2026)
6. Figma Help, resumo de busca sobre ferramenta Select/Edit — https://help.figma.com/hc/en-us/articles/42009840449175 (13/11/2026)
7. Fórum Figma, relatos Point and Edit (Secundário) — https://forum.figma.com/report-a-problem-6/figma-make-point-and-edit-changing-page-50243 (13/11/2026)
8. Figma Help, edição e histórico de versões do Make — https://help.figma.com/hc/en-us/articles/42009840449175 (13/11/2026)
9. v0, "Design mode" — https://v0.app/docs/design-mode (13/11/2026)
10. Comunidade Vercel (Secundário) — https://community.vercel.com/t/why-v0-design-mode-applies-unsolicited-changes-to-surrounding-elements-during-targeted-edits/36673 (13/11/2026)
11. v0 API, "Design mode" — https://v0.app/docs/api/v2/guides/design-mode (13/11/2026; lido apenas via resumo de busca)
12. Lovable Docs, "Preview toolbar" — https://docs.lovable.dev/features/preview-toolbar (13/11/2026)
13. Lovable Docs, "Design" — https://docs.lovable.dev/features/design (13/11/2026)
14. Lovable Docs, "History" — https://docs.lovable.dev/features/projects/history (13/11/2026)
15. Bolt Support, "Visual edits" — https://support.bolt.new/building/visual-edits (13/11/2026)
16. Subframe Docs, "Ask AI" — https://docs.subframe.com/learn/ask-ai (13/11/2026)
17. Subframe Docs, "Making quick edits" — https://docs.subframe.com/learn/ask-ai/making-quick-edits (13/11/2026)
18. Subframe Docs, "Prompt to design" — https://docs.subframe.com/learn/ask-ai/prompt-to-design (13/11/2026; só resumo de busca)
19. Google Developers Blog, "Introducing Stitch" — https://developers.googleblog.com/en/stitch-a-new-way-to-design-uis/ (13/11/2026)
20. Tutorial de terceiros (Secundário) — https://it.aad.vt.edu/howto/google-stitch/ (13/11/2026)
21. Fórum Google AI (Secundário) — https://discuss.ai.google.dev/t/exporting-stitch-to-figma/104903 (13/11/2026)
22. html.to.design (Secundário) — https://html.to.design/blog/from-google-stitch-to-figma/ (13/11/2026)
23. Cursor Docs, "Design Mode" — https://cursor.com/docs/agent/design-mode (13/11/2026)
24. Cursor Blog, "Browser visual editor" (11/12/2025) — https://cursor.com/blog/browser-visual-editor (13/11/2026)
25. Terceiro sobre Cursor 3 (Secundário) — https://pasqualepillitteri.it/en/news/3794/cursor-design-mode-what-it-is-how-it-works-cursor-3 (13/11/2026)
26. Penpot Help, "Penpot MCP server" — https://help.penpot.app/mcp/ (13/11/2026)
27. Penpot Blog, fluxos multidirecionais de IA — https://penpot.app/blog/guide-to-multi-directional-ai-workflows/ (13/11/2026)
28. Builder.io Docs, "Visual Editor AI" — https://www.builder.io/c/docs/visual-editor-ai (13/11/2026)
29. Builder.io Docs, "Get started Fusion" e prompt essentials — https://www.builder.io/c/docs/get-started-fusion (13/11/2026; só resumo de busca)
30. Onlook, repositório — https://github.com/onlook-dev/onlook (13/11/2026; só resumo de busca)
31. Framer Community, guia do Agent 3.0 (Secundário) — https://www.framer.com/community/posts/LaHu5HSaMFBDKGJ6kg8KeP/ (13/11/2026; só resumo de busca)
32. Framer Marketplace, Context Picker (Secundário) — https://www.framer.com/community/marketplace/plugins/context-picker/ (13/11/2026; só resumo de busca)
33. Framer, glossário "Agent branch" — https://www.framer.com/ja/dictionary/agent-branch (13/11/2026; página sem conteúdo útil)

## 4. Padrões recorrentes

- **Selecionar no preview e anexar ao chat** (Lovable, Bolt, Cursor, Framer, Figma Make, v0). O chat vira canal de execução, não interface principal.
- **Dois caminhos por tipo de pedido:** controle direto para propriedades (cor, espaço, texto) e IA para estrutura (v0, Bolt, Figma Make, Claude Design).
- **Edições em stage ou em lote antes de aplicar** (v0 Apply, Bolt Save, Figma Make, Cursor Batch). Reduz custo e dá controle.
- **Reversão por versão do projeto inteiro** (Lovable, Figma Make, Bolt). Granularidade fina só antes de aplicar.
- **Variações lado a lado com Keep/Discard** (Subframe). É o único caso documentado de escolher entre alternativas dentro do canvas.
- **Fila de pedidos durante a execução** (Lovable, Figma Make, Subframe).
- **Custo como fricção:** prévia grátis, aplicar custa (Bolt, Figma Make, Lovable).
- **Código como fonte de verdade.** Quase todas traduzem edição visual em código via modelo, sem garantia determinística (v0 API doc [11]). Isso explica efeitos colaterais em vizinhos.

## 5. Lacunas que nenhuma ferramenta resolve

| Lacuna | Veredito | Evidência |
|---|---|---|
| Origem por camada IA × humano | **Confirmada** nas fontes oficiais lidas. Nenhuma marca autoria por camada ou propriedade. | Claude Design, v0, Lovable, Bolt, Subframe e Cursor: [NE]. Versões existem, mas sem autoria por elemento. |
| Proteção de ajustes manuais quando a IA refaz | **Confirmada.** Claude Design admite que não explica a interação [3]. v0 tem relato de alteração de vizinhos [10]. | Nenhum mecanismo de "bloquear" camada ou propriedade documentado. |
| Checagem de DS antes da revisão humana | **Parcialmente derrubada:** existe uso de tokens e DS como insumo (Cursor [24], Builder [28], Claude Design importa DS [3]). **Não** há checagem de conformidade antes da revisão. | Insumo ≠ validação. Nenhuma fonte descreve relatório de desvio. |
| Desfazer só da IA sem perder edições manuais posteriores | **Confirmada.** Lovable diz que não há reversão parcial [14]. Bolt não desfaz item salvo [15]. Cursor Cmd+Z não desfaz o agente [23]. Subframe Cmd+Z não cobre MCP/API [16]. | Reversão é por versão inteira. |

Ressalva: "não encontrado" significa ausente nas páginas oficiais lidas, não prova de inexistência.

## 6. Implicações para as 3 direções do Eureka

### A. Pedido na seleção com preview fantasma
- **Vantagens:** casa com o padrão dominante (selecionar e pedir). v0 e Bolt validam o "pendente antes de aplicar" e o toggle antes/depois [9][15]. Mantém a edição visual no centro.
- **Limitações:** nenhuma ferramenta mostra preview fantasma de escopo estrutural fora do elemento. Risco de o escopo real passar da seleção (relato em v0 [10]). Chat de fallback ainda necessário.
- **Testar com designers:** se entendem o escopo do fantasma; se querem aceitar parcial; se a seleção perdida entre turnos incomoda; tempo de espera aceitável.

### B. Copiloto com cartões de proposta e diff por item
- **Vantagens:** preenche a lacuna de aprovação granular. Bolt (lote revisável) e Figma Make (stage por item) são os precedentes mais próximos [5][15]. Permite carregar origem IA × humano por item.
- **Limitações:** nenhuma ferramenta tem diff visual por item no canvas. Cartões podem puxar o foco para o painel e aumentar a dependência de chat. Custo de construir diff de camadas.
- **Testar:** cartões em lista versus marcadores no canvas; se o designer aceita revisar muitos itens; nível de agrupamento do lote.

### C. Quadro de exploração com 3–5 alternativas
- **Vantagens:** Subframe prova Keep/Discard, comparação com o original e mistura de elementos entre variações [16][17]. Stitch indica 2–5 variações, só por fontes secundárias [20]. Boa para fase divergente.
- **Limitações:** pouca evidência sobre como alternativas voltam ao fluxo de edição. Custo de geração multiplicado. Subframe é inconsistente (1–4 versus até 3) [16][17]. Não resolve ajuste fino.
- **Testar:** número ideal de alternativas; se o designer mistura partes ou escolhe uma inteira; como volta ao canvas principal sem perder edições.

### O que o benchmark sugere testar em conjunto
Granularidade de aprovação (item, grupo, lote); sinal visual de origem IA × humano; desfazer só da IA; efeito de bloquear camadas editadas à mão; uso do teclado e do leitor de tela (Claude Design é o único parâmetro [3]).

## 7. O que não foi verificado

- Aprovação, diff e proteção de edições manuais no Claude Design (suporte silencia) e a existência de "Tweaks" como recurso oficial.
- "Checkpoint por turno" e "perder a seleção ao enviar" no Figma Make.
- Variações por eixo e perda de componentes na exportação do Stitch; os limites de Standard/Rapid vêm só de fórum. Não encontrei documentação oficial de Stitch além do post de lançamento.
- Seleção de elemento no Builder.io (páginas divergentes) e todo o comportamento de Onlook e Framer (só resumos de busca e fontes comunitárias).
- Diff de código do agente no Cursor; datas de Cursor 3 e Canvases (só terceiros).
- Acessibilidade em todas as ferramentas, exceto Claude Design.
- Fontes 6, 11, 18, 29 a 32 foram lidas só por resumo de busca, não por leitura da página inteira.
- Este benchmark cobre documentação, não uso prático. Nenhum protótipo foi executado.
