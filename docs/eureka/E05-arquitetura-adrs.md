# E05 — Plano de arquitetura, ADRs e contratos de interface

Card Notion E05 · Eureka Design · 13/11/2026 · autor: Integration Lead (Claude) · **rascunho para revisão da dona do produto**.
Critério de aceite do card: arquitetura aprovada pela designer; plano de spikes e riscos priorizados.
Nada aqui foi implementado. Base: relatórios E01–E04, `HANDOFF.md`, skill `eureka-ux-ai` e `AGENTS.md`.

Legenda de status dos ADRs:
- **Aceita (dona do produto, 13/11)**: decidido; não reabrir.
- **Aceita, sujeita a teste**: decidido, mas pode mudar depois do teste da dona.
- **Proposta**: recomendação deste documento; precisa de decisão.
- **Pendente da dona**: há alternativas abertas; recomendação indicada.

## 1. Resumo

1. O Claude orquestra **fora do app**, por MCP. O Eureka (fork do FigCraft) é a superfície visual e o motor de edição.
2. Figma Variables são a única fonte de tokens. O Eureka pode guardar um **snapshot somente leitura**, com data e hash, gerado do Figma.
3. O modelo de edição é o nativo do FigCraft. TSX é saída. Importação de TSX só depois de um spike limitado.
4. Peças novas no engine: **propostas** (dry-run → diff → aceitar/rejeitar), **delta** "o que mudou", **origem por camada**, **lint de DS** e **adaptador de tokens**.
5. Conectores são **capacidades com contrato** (entrada, saída, permissão, confirmação, custo). O `ds-orchestrator` é candidato a ponto de entrada, sem acoplar o Eureka a ele.
6. Código do Eureka fica isolado; arquivos do upstream são tocados só com mudanças aditivas e pequenas.
7. O ADR-004 (propostas) só fecha com o resultado do spike do Gate 0.

## 2. Diagrama de componentes

```mermaid
flowchart LR
  D[Designer] -- edita no canvas, pede na seleção, revisa cartões --> APP

  subgraph APP[Eureka app (fork FigCraft, Rust/egui)]
    UI[UI: pedido na seleção · quadro de exploração · cartão de proposta · selo Gatekeeper]
    ENG[Engine FigCraft: comandos, command.batch, undo, variables]
    subgraph NOVO[Peças novas]
      P[proposal.* / dryRun<br/>dry-run, diff, aceite com guarda]
      DL[document.changed + delta<br/>o que mudou desde o último turno]
      OR[Origem por camada<br/>pluginData eureka.origin]
      LT[Lint de DS<br/>leitura]
      TA[Adaptador de tokens<br/>snapshot Figma → variable.import]
    end
    UI --> ENG
    ENG --- NOVO
  end

  CC[Claude: orquestrador<br/>Claude Code / Desktop, sessão principal] -- MCP stdio, figcraft-cli mcp --attach --> APP
  APP -. canal de controle 127.0.0.1 .- CC

  CC -- MCP HTTP, leitura; escrita só com confirmação --> FG[Figma MCP<br/>Variables, componentes publicados]
  CC -- MCP HTTP, leitura; escrita só com confirmação --> NT[Notion MCP<br/>regras, Registro, rodadas]
  CC -- capacidade pre_aprovar_ds, auditar... --> DS[ds-orchestrator / DS Experts Studio<br/>plugins do Claude Code]
  CC -- capacidade propor_alternativas --> GP[gerador-propostas Naty<br/>+ perfis references/marcas]
  CC -. só relatório de compatibilidade .-> LIB[Biblioteca de código<br/>@soma-monorepo/core-components]
  FG -- snapshot com data e hash --> CACHE[(Cache de tokens<br/>fora do repo, somente leitura)]
  CACHE -- variable.import --> TA
```

Regras do diagrama:
- O Eureka **não** fala com Figma, Notion, GitHub nem Artifacts. Quem fala é o orquestrador (E04, "Arquitetura de conexão").
- Toda escrita externa (Notion, Figma, publicar, merge, DS) passa por confirmação humana.
- Agentes externos continuam independentes. O orquestrador chama a capacidade e repassa o resultado.

## 3. ADRs

### ADR-001 — Orquestração fora do app via MCP (chat nativo depois)

- **Contexto.** Plugins do FigCraft rodam em WASM sem rede (E01, `docs/plugins.md:8-10`). Não há cliente LLM no app; rede só no L6 (`xtask/src/layers.rs:30`). O servidor MCP já existe, com 19 ferramentas e `run_command` para qualquer comando (`crates/mcp/src/tools.rs:95-115`). Os agentes da Naty e do DS são plugins do Claude Code e só podem ser chamados pela sessão principal (E04).
- **Decisão.** No MVP, o Claude (Code ou Desktop) é o orquestrador e fala com o Eureka por `figcraft-cli mcp --attach PORT`. Chat nativo no app fica para depois.
- **Alternativas.** (a) Painel de chat nativo no app: exige chave de API, cliente HTTPS e dependência nova; conflita com "Rust only, sem webview" se for webview (`AGENTS.md`, regra Rust 4). (b) Orquestrador com Agent SDK: perde a ferramenta Artifact e o prompt interativo de aprovação (E04).
- **Consequências.** Zero mudança no upstream para conversar. A UI do app mostra estado e coleta decisão; a conversa fica no Claude. Precisa de gancho de aprovação se um dia virar SDK.
- **Status.** Aceita (dona do produto, 13/11) — interação A + C + cartão de B; **Aceita, sujeita a teste** do protótipo https://claude.ai/artifact/55hRUrmJ6LrHTFxz3jbeDo.
- **Evidência.** E01 "Capacidades aproveitáveis" e "Lacunas" (cliente LLM: G); E04 "Arquitetura de conexão recomendada para o MVP"; `docs/mcp.md:16-20`; skill `eureka-ux-ai` §1.

### ADR-002 — Fonte de tokens e cache

- **Contexto.** A fonte é a coleção **Tokens WEB** do DS N Marcas (`FsU6PD5PxodVk0IVMWQrLP`, key `b804c201…0bf0`): 312 variables, 12 modos de marca; mais a coleção **Layout** (14 variables, Desktop/Mobile). Há quatro cópias de tokens fora do Figma, todas fotos sem sincronização; uma vem do código de Tecnologia (E02, "Cópias encontradas"). `variable.import` lê só o JSON de variables do Figma (E01, `cmd/variables_io.rs:26-29`). REST de variables exige Enterprise com assento Full; o caminho viável é o Figma MCP, com cota diária (E04).
- **Decisão (aceita).** Figma Variables são a fonte oficial. O Eureka pode usar snapshot/cache para performance, **somente leitura, com data e hash, gerado do Figma**. Nunca segunda fonte da verdade.
- **Detalhes propostos:**
  - **Formato.** O JSON de local variables do Figma (`meta.variableCollections`, `meta.variables`, `valuesByMode`, `resolvedType`, `scopes`, `key`, aliases `VARIABLE_ALIAS`), que o `variable.import` lê sem conversor. Mais um bloco `eureka_snapshot` com `source`, `fileKey`, `collection`, `collectionKey`, `modes`, `subset`, `exportedAt`, `contentHash`. O exemplo existente segue isso: `/Users/fernanda.caroline/Claude/eureka-cache/tokens-web-animale-farm-9d7af6d863af.json` (23 variables, Animale e Farm, `exportedAt 2026-11-13T15:51:34Z`).
  - **Onde fica. Recomendação: fora do repo do Eureka**, numa pasta de cache local (`~/Claude/eureka-cache/` hoje) e na raiz de leitura do app (`--automation-read-root`). Motivos: o repo não deve manter cópia de material do DS (HANDOFF, "Como reproduzir"); um arquivo versionado no git tende a virar fonte. Alternativa: versionar só um **manifesto** (fileKey, collectionKey, modos, hash, data), sem valores, para testes reproduzirem "qual snapshot usei". Testes automatizados usam fixture **sintética**, não tokens reais. → **D2 aceita pela dona em 13/11: cache fora do repo; no repo só manifesto sem valores; testes com fixture sintética.**
  - **Invalidação.** Nome do arquivo e manifesto com hash (recomendo SHA-256 completo no manifesto; o exemplo usa 12 caracteres, suficiente para nome, fraco para conferência). Regra: ao iniciar uma sessão, o orquestrador compara o hash do snapshot com uma leitura leve do Figma (contagem + data da última publicação, quando legível); diferente = gerar novo snapshot. Sem acesso ao Figma = usar o snapshot com **aviso visível** ("tokens de DD/MM, não confirmados"). Validade padrão sugerida: 7 dias; vencido = aviso, nunca bloqueio silencioso. Versão publicada da biblioteca: **não sabemos ler hoje** (E02, pergunta aberta).
  - **Business Rule e Components na mesma coleção.** Tokens WEB tem `Business Rule/*` (22 string, 17 boolean) e `Components/*` (41 string, 77 boolean, 5 float) junto dos visuais (E02). O adaptador separa por **primeiro segmento do nome** em três grupos: `visual` (Colors, Spacing, Font, Border…), `business_rule` e `component_config`. O lint e o gerador só usam `visual` para estilo. `business_rule` entra como texto padrão de marca (memória: textos padrão por token Business Rule). `component_config` nunca vira copy. Importação para o Eureka: só `visual` por padrão; os outros sob pedido.
  - **Sem `codeSyntax`.** 0 de 326 variables têm `codeSyntax` (E02). O nome CSS é **derivado** (minúsculas, `/` e espaço → `-`) e marcado `derivado: true`. Não pedir ao DS para criar `codeSyntax` como parte deste projeto (não alterar DS); registrar como sugestão para a liderança.
  - **Escopos.** `Spacing/*` tem scope só `GAP` (exemplo: `Spacing/sp-md`, no cache acima); `Spacing/Mobile/sp-productcard-gap` tem `WIDTH_HEIGHT` e `GAP`. Preservar `scopes` no snapshot. O lint trata escopo como **informativo**: padding com variable de escopo `GAP` gera aviso "uso fora do escopo declarado", não erro, até a liderança definir a regra. Não "corrigir" escopos no DS.
  - **Aliases.** O exemplo tem `aliasesResolved: true`. Isso perde a cadeia semântica (ex.: token de marca → primitivo). Recomendo guardar o alias **e** o valor resolvido, para o lint saber o token certo e não só a cor certa.
- **Alternativas.** (a) Ler sempre ao vivo no Figma: sem cópia, mas gasta cota (200–600 leituras/dia) e falha offline. (b) Usar `tokens.json` do artifact DS Animale: formato próprio, um modo, cópia de 27/10 — **rejeitada** pela decisão 2. (c) DTCG/Tokens Studio: o FigCraft não lê (E01).
- **Consequências.** Adaptador fica fora do upstream (conversor + `variable.import`), zero arquivo do upstream tocado. Precisa de um leitor no Figma (script `use_figma` somente leitura) com cota controlada.
- **Status.** Aceita (dona do produto, 13/11) para "Figma é fonte; cache somente leitura". **Proposta** para local do cache, invalidação, separação por grupo e escopos.
- **Evidência.** E02 "Mapa de autoridade", "Riscos 3 e 5", "Contrato preliminar"; E04 "Tokens: caminho mais curto"; E01 `variables_io.rs:26-29`, `format/src/rest/variables.rs:114,233`.

### ADR-003 — Modelo de edição nativo + export TSX + spike de importação limitada

- **Contexto.** O documento do FigCraft é `imbl::OrdMap<NodeId, Arc<Node>>`, com undo por snapshot e ~575–640 comandos (E01). O codegen já exporta React, Tailwind, Vue e outros (`engine/src/dev/codegen.rs:44,51`). Importar HTML/React como camadas editáveis não foi verificado (E04).
- **Decisão.** O modelo nativo do FigCraft é o modelo de edição. TSX é **saída** (codegen + Code Connect). Spike de importação limitada.
- **Escopo do spike de importação.** Só um subconjunto: JSX com instâncias de componentes mapeados por Code Connect, auto layout (flex) e tokens por nome CSS derivado. Sem CSS arbitrário, sem lógica, sem estado.
- **Critério para ampliar a importação** (todos):
  1. Ida e volta (Eureka → TSX → Eureka) em 10 telas reais do Animale sem perda de vínculo a token ou componente.
  2. Nada importado vira componente novo ou cópia desenhada; peça sem mapeamento vira aviso.
  3. Custo estimado da versão ampla ≤ M e sem tocar mais arquivos do upstream que o MVP.
  4. Caso de uso confirmado pela dona (ex.: trazer protótipo do Claude Design).
- **Alternativas.** (a) TSX como fonte da verdade: dois modelos para sincronizar; conflita com "Design é autoridade". (b) Sem importação: mais simples; fecha a porta para protótipos em código.
- **Consequências.** "Código sempre consistente" = código **gerado** do documento, nunca editado à mão como fonte. Código de Tecnologia só como relatório de compatibilidade.
- **Status.** Aceita (dona do produto, 13/11). Critério de ampliação: **Proposta**.
- **Evidência.** E01 "Codegen", "Code Connect"; E04 "Artifacts: fallback de importação"; E02 linha "Código de Tecnologia".

### ADR-004 — Propostas: dry-run + aceite com guarda + delta "o que mudou"

- **Contexto.** `command.batch` aplica direto, sem dry-run (`cmd/edit.rs:535`). O rollback de `grouped` restaura documento, seleção, histórico e alocador de ids (`session/undo.rs:322-387`). Diff por nó já existe (`doc/src/lib.rs:416-428`). Eventos `document.*` estão reservados mas não emitem (`tools.rs:383`). O benchmark mostra que preview e aprovação antes de aplicar são raros e valorizados (E03 §1).
- **Decisão proposta.**
  - **Passo 0:** `dryRun: true` em `command.batch` (roda em `grouped`, calcula diff, força rollback) e `ifUnchanged: <hash dos nós tocados>` no aceite. Estado da proposta fica no orquestrador.
  - **Passo 1 (se o passo 0 passar):** comandos `proposal.create/diff/accept/reject` com estado na `Session` (E01, "Spike recomendado").
  - **Delta:** emitir `document.changed` com `changed_ids` ao fim de `execute` (`session/exec.rs`), agregando gestos de arrastar; o orquestrador pede "o que mudou desde o turno N".
  - **Guardas:** proibido dentro de proposta: `file.*`, `online.*`, `live.*`, `plugin.*`, área de transferência. Propostas desligadas em documento colaborativo no MVP.
  - **Cartão de proposta (B)** é a única superfície de revisão, usada por A e C.
- **Alternativas.** (a) Aplicar e confiar no ⌘Z (estado atual do spike): não mostra antes. (b) Cópia do documento em outra aba: pesado e confunde.
- **Consequências.** 2 arquivos do upstream no passo 0 (`edit.rs`, `undo.rs`); 3 + 1 novo no passo 1; 1 (`exec.rs`) para eventos.
- **Status.** **Passo 0 validado no spike** (13/11). Passo 1 (`proposal.*` com estado) e evento `document.changed` seguem como **Proposta** para o G1.
- **Critério de sucesso do spike** (E01): em 20 batches reais, `create → reject` deixa o `Document` igual (`PartialEq`) e o `undo_depth` igual; `create → accept` soma exatamente 1; nó editado pelo designer entre criar e aceitar = `stale`.
- **Resultado do spike (13/11/2026, Editor Engineer · Sonnet; revisão Integration Lead · Opus; commit `e8f06439`):**
  - Implementado: `command.batch {dryRun}` (rollback total, diff por propriedade, `baseHash` FNV-64 dos nós/variables tocados), `command.batch {ifUnchanged}` (aplica em 1 passo de desfazer ou falha `stale` com os conflitos), `document.checkpoint` / `document.changesSince` (até 8 snapshots), `proposal.preview` / `proposal.previewEnd` (canvas só leitura via `start_preview`). Código novo em `cmd/eureka.rs`; upstream: só linhas de registro em `cmd.rs`, `edit.rs`, `lib.rs`, `session/new.rs`, `session/exec.rs`, `tests.rs`, `mcp/tools.rs`.
  - Testes: 7 novos (dry-run sem rastro, aceite = +1 desfazer, `stale` em nó tocado e aceite em nó não tocado, delta de texto/cor/vínculo, cursor inválido, preview só leitura) + sweep de parâmetros; 66 testes do engine e 18 do MCP passaram; fmt, clippy (engine, mcp) e `xtask layers` ok. **Não rodados:** `xtask ci`, wasm, suíte completa.
  - Demo no app real com snapshot do Figma (Tokens WEB, Animale/Farm): gerar → editar → delta → preview → aceitar (textos do designer preservados) → desfazer → `stale` → troca de marca. Evidência em `docs/eureka/demo-s1/`. Dry-run: 1,9 ms em 7 nós.
  - **Critério de sucesso parcialmente atendido:** os invariantes foram provados em testes, mas não em 20 batches reais; medir desempenho em documento grande fica para o G1.
  - Limitações: o engine não aplica escopo de variable (ver D4); diff com ruído de layout (`transform`, `width`, `height`, `children`); preview reaproveita a faixa de "versão"; dry-runs lembrados só em memória (16); ids criados no aceite podem diferir do preview se o designer criar nós no meio.
  - Rollback sem rastro: sim / não — evidência:
  - Aceitar = 1 desfazer: sim / não — evidência:
  - `stale` detectado: sim / não — evidência:
  - Desempenho (tempo de dry-run em documento real):
  - Efeitos fora do documento encontrados:
  - Decisão final da dona:
- **Evidência.** E01 "Lacunas", "Spike recomendado", "3 riscos críticos" (risco 3); HANDOFF "Teste do ciclo híbrido"; E03 §1 itens 2–4.

### ADR-005 — Convivência com o upstream

- **Contexto.** Licença MIT OU Apache-2.0 permite o fork (E01). Marcas ArtCraft não são livres (`docs/brand/LICENSE-brand.txt:1-8`). Registro de comandos é lista fixa no engine (`cmd.rs:73-132`): crate L6 não registra comando. CI chama `storytold/craft-common-actions@main`, que pode não rodar no fork. O bloco de regras do `AGENTS.md` tem hash checado por `house-rules.yml`. Regra 15: PR não diz que agente escreveu.
- **Decisão (aceita).** Preservar compatibilidade sempre que possível; customizações isoladas; testes próprios.
- **Detalhes propostos:**
  - **Onde vive o código.** Lógica pura (diff de proposta, lint, conversor de tokens, contrato de conector) em crate novo `crates/eureka-core` (L4–L5, sem egui, sem rede). UI em crate/módulo L6. Comandos que precisam do engine: arquivo novo `cmd/eureka_*.rs` + 1 linha de registro em `cmd.rs`. Documentação e regras em `docs/eureka/` e `.claude/`.
  - **Arquivos do upstream tocados (alvo do MVP: 4 a 8, aditivos):** `cmd.rs`, `cmd/edit.rs`, `session/undo.rs`, `session/exec.rs`, `xtask/src/layers.rs` (listar crates novos, senão ficam sem fiscalização: `layers.rs:43`), `ui-egui/src/lib.rs`, e rebranding (`apps/figcraft/build.rs`, `ui-egui/src/dialogs.rs`, `README`).
  - **Sincronização.** Branch `upstream-sync` rebaseado com frequência (sugestão: semanal); patches pequenos; teste de não-regressão a cada sync. Espelhar `storytold/craft-libs` na org e conferir licenças (lista em `AGENTS.md:34-48`).
  - **CI.** Recomendo CI **próprio do Eureka** em workflow separado (`cargo xtask ci` + testes do Eureka), sem editar os workflows do upstream. Os workflows do upstream ficam como estão (podem falhar no fork). Alternativa: pedir acesso ao `craft-common-actions`.
  - **Regra 15.** PR para o upstream: sem atribuição de agente. No fork: livre (decisão da dona no E01). Regras do Eureka nunca no bloco com hash do `AGENTS.md`.
  - **Rebranding ArtCraft:** obrigatório antes de distribuir (nome, logo, "Sobre", metadados do executável, links). Não editar os `.md` de marca do upstream; substituir assets e registrar em `ATTRIBUTION.md`. Desligar `craft-update` (hipótese: consulta releases do upstream).
- **D5 aceita (13/11): workflow próprio do Eureka, sem mexer nos do upstream.** **Ainda precisa de decisão:** D6 (nome/marca do produto), D7 (Eureka contribui de volta ao upstream ou só consome?).
- **Status.** Aceita (dona do produto, 13/11) no princípio. **Pendente da dona** nos itens acima.
- **Evidência.** E01 "Licença e dependências", "Extensibilidade e convivência", "3 riscos críticos" (risco 2); `AGENTS.md` regras 11 e 15.

### ADR-006 — Contrato de conectores por capacidade

- **Contexto.** DS Experts Studio = seis especialidades; `ds-orchestrator` = as seis + `ds-workflow` (E04). Os agentes restritos não chamam outros agentes. O contrato da etapa do orchestrator é `stage.schema.json`. A matriz do Gatekeeper ainda é piloto (E02, risco 8).
- **Decisão (aceita).** DS Experts Studio como **conjunto de capacidades**; avaliar `ds-orchestrator` como ponto de entrada; não acoplar o Eureka a um agente específico.
- **Detalhes propostos.** O Eureka conhece **verbos**, não agentes: `ler_tokens`, `ler_componentes`, `auditar_ds`, `pre_aprovar_ds`, `propor_alternativas`, `documentar`, `registrar_notion`, `publicar_pagina`, `relatorio_compat_codigo`. Um **registro de conectores** (arquivo de configuração no orquestrador) liga verbo → implementação (skill, MCP, arquivo). Trocar o Gatekeeper é trocar uma linha do registro. Campos do contrato: os de E04 ("Rascunho: campos de um contrato de conector"). Contrato na seção 4.
- **ds-orchestrator como entrada.** Candidato para `pre_aprovar_ds` e `auditar_ds` via `ds-workflow`. Vantagem: já traz guardrails G01–G12 e precedência Notion > Figma. Risco: amarra o Eureka ao ritmo e à versão (0.2.13) do plugin. Recomendo: usar as skills individuais (`ds-gatekeeper`, `ds-audit`) por capacidade no MVP, e o `ds-workflow` só para demandas de várias etapas.
- **Confirmação humana.** Toda capacidade com escrita externa declara `acoes_com_confirmacao`. O orquestrador não usa `allowedTools` para escrita (E04, pergunta 7).
- **Status.** Aceita (dona do produto, 13/11) no princípio. Registro e verbos: **Proposta**.
- **Evidência.** E04 "Matriz", "Como acionar o ds-orchestrator", "Rascunho: campos"; E02 "Regras operacionais dos agentes".

### ADR-007 — Perfis de marca consumidos do gerador-propostas

- **Contexto.** Perfis vivem em `Soma-UXDesign/gerador-propostas`, `plugins/gerador-propostas/skills/gerador-propostas/references/marcas/` (`_modelo.md`, `nv.md` parcial, dona Nataly; `brand-registry.yaml`). Repo privado. O HANDOFF dizia que não existiam; E04 corrigiu.
- **Decisão (aceita).** Consumir a fonte oficial da Naty, sem duplicar arquivos.
- **Detalhes propostos.**
  - Leitura pelo GitHub (`gh api` com a conta da pessoa, somente leitura) ou pelo plugin instalado (a skill já carrega as referências). Guardar no contexto da sessão: caminho, SHA do commit e data. Nunca gravar cópia no repo do Eureka.
  - **Sem perfil** (hoje: todas as marcas menos NV): trabalhar só com usabilidade e DS; o cartão mostra "sem perfil de marca" e quem é a dona sugerida. Perfil parcial: usar o que existe e listar campos vazios.
  - Sem acesso ao repo: mesmo comportamento de "sem perfil", com aviso de acesso.
- **Status.** Aceita (dona do produto, 13/11). Mecanismo de leitura: **Proposta**.
- **Evidência.** E02 linha "Perfis de marca"; E04 "Naty e 007"; HANDOFF "Perfis de marca" (desatualizado).

### ADR-008 — Segurança do canal local

- **Contexto.** Canal de controle e `mcp --attach` só em `127.0.0.1`, **sem autenticação** (E01, `control_server.rs:96-115`). Qualquer processo local que saiba a porta executa qualquer comando, inclusive ler/salvar arquivos e, com rede, publicar (`online.*`, `live.*`). Confinamento de arquivos é opt-in (`--automation-read-root/--automation-write-root`). Nome do agente é auto-declarado.
- **Recomendação para o MVP:**
  1. Iniciar o app **sempre** com raízes de leitura e escrita confinadas (pasta do projeto Eureka + cache de tokens). Sem mudança no upstream.
  2. Porta aleatória por sessão, gravada num arquivo com permissão só do usuário.
  3. **Token por conexão** (segredo no mesmo arquivo; primeira linha do canal) antes de qualquer uso fora de teste. Toca `control_server.rs` (1 arquivo do upstream) ou um proxy autenticado (0 arquivos).
  4. Tratar origem "Claude (agent)" como informativa; o Gatekeeper nunca depende dela.
  5. Desligar `online.*`/`live.*` no MVP.
- **Alternativas.** (a) Aceitar o risco no MVP só em máquina pessoal (mais rápido). (b) Proxy autenticado fora do app (zero upstream, mais uma peça). (c) Token no `control_server.rs` (simples, um patch no upstream; bom candidato a contribuir de volta).
- **Status.** **Aceita (dona do produto, 13/11)** conforme a recomendação: 1, 2, 4 e 5 já no MVP; token (c) antes de outro designer usar.
- **Evidência.** E01 "Segurança", "3 riscos críticos" (risco 1); E04 pergunta 4.

### ADR-009 — Origem por camada (IA × humano) e proteção de edição manual

- **Contexto.** Nada no `Node` registra autor (`doc/src/node.rs:449+`). `Session.agent` só existe durante o comando. Há `pluginData` livre por camada (`props.rs:1108`). Nenhuma ferramenta do benchmark tem origem por camada nem "desfazer só a IA" (E03 §1). É diferencial do Eureka.
- **Decisão proposta.**
  - **Como guardar.** `pluginData["eureka.origin"]` gravado no mesmo batch que cria ou altera a camada: `{autor: "ia"|"humano", agente, proposta_id, turno, props_tocadas, hash_props}`. Sem mudar o formato do arquivo.
  - **Quando vira humano.** Com o evento `document.changed` (ADR-004), o engine marca `humano` nas **propriedades** editadas pelo designer (não na camada inteira).
  - **Proteção.** Ao gerar nova proposta, propriedade marcada `humano` não é sobrescrita; se a IA precisar mudar, o cartão pede confirmação explícita ("isso desfaz sua edição de texto").
  - **Limites.** Origem é **informativa, não prova** (nome do agente é auto-declarado; qualquer cliente local escreve `pluginData`). Edição feita fora do Eureka (ex.: `.fig` reimportado) perde a marca. Colaboração ao vivo fora do MVP. Não usar origem para auditoria ou bloqueio de governança.
- **Alternativas.** (a) Campo `author` no `Node`: muda formato e muitos arquivos do upstream. (b) Diário de origem fora do documento: sai de sincronia com undo.
- **Status.** **Proposta** (depende do evento do ADR-004).
- **Evidência.** E01 "Lacunas" (origem; esforço M), "Segurança" (nome auto-declarado); E03 §1 item 3; HANDOFF "Benchmark".

## 4. Contratos de interface (rascunho, não implementar)

### 4.1 Proposta

Pedido de dry-run (passo 0, sobre `command.batch`):

```json
{
  "command": "command.batch",
  "params": {
    "dryRun": true,
    "name": "Ajustar card ao DS (Animale)",
    "commands": [
      {"command": "node.set", "params": {"id": "12:4", "fills": [{"variable": "Colors/Surface/Brand/Background"}]}},
      {"command": "node.set", "params": {"id": "12:9", "pluginData": {"eureka.origin": "{\"autor\":\"ia\",\"turno\":7}"}}}
    ]
  }
}
```

Resultado do dry-run (diff):

```json
{
  "proposalId": "p-7f3a",
  "base": {"hash": "sha256:…", "touched": ["12:4", "12:9"]},
  "changes": [
    {"id": "12:4", "name": "Botão", "props": {"fills": [{"hex": "#7A4B2A"}, {"variable": "Colors/Surface/Brand/Background"}]}}
  ],
  "created": [], "deleted": [],
  "variables": [],
  "warnings": [{"code": "scope_outside", "id": "12:4", "msg": "Spacing/sp-md tem escopo GAP; usado em padding"}],
  "lint": {"errors": 0, "warnings": 1},
  "protectedHuman": [{"id": "12:7", "prop": "characters", "action": "preservado"}]
}
```

Aceite com guarda:

```json
{"command": "command.batch", "params": {"ifUnchanged": "sha256:…", "name": "Ajustar card ao DS (Animale)", "commands": ["…mesmos passos…"]}}
```

Respostas possíveis do aceite:

```json
{"status": "applied", "proposalId": "p-7f3a", "undoSteps": 1, "touched": ["12:4", "12:9"]}
{"status": "stale", "proposalId": "p-7f3a", "changedSince": ["12:4"], "hint": "refazer dry-run"}
{"status": "rejected", "proposalId": "p-7f3a"}
{"status": "error", "proposalId": "p-7f3a", "step": 2, "message": "…", "rolledBack": true}
```

Estados do cartão: `rascunho → em_revisao → aceita | rejeitada | stale | erro`. `stale` volta a `rascunho` com novo dry-run.

### 4.2 Delta "o que mudou"

```json
{
  "topic": "document.changed",
  "sinceTurn": 6,
  "untilTurn": 7,
  "changes": [
    {"id": "12:7", "name": "Título", "props": ["characters"], "by": "humano"},
    {"id": "12:4", "name": "Botão", "props": ["fills"], "by": "humano", "lint": ["hex_sem_token"]}
  ],
  "created": ["12:30"],
  "deleted": [],
  "truncated": false
}
```

Regras: gestos contínuos (arrastar) viram uma mudança; `truncated: true` acima de um limite e o orquestrador relê o documento.

### 4.3 Conector (pedido / resultado / estado)

Declaração (registro de conectores):

```json
{
  "id": "gatekeeper",
  "versao_contrato": "0.1",
  "capacidade": "pre_aprovar_ds",
  "implementacao": {"interface": "skill", "alvo": "ds-orchestrator:ds-gatekeeper", "versao": "0.2.13"},
  "permissoes": {"leitura": ["figma", "notion"], "escrita": [], "rede": true},
  "acoes_com_confirmacao": [],
  "limites": {"timeout_s": 600},
  "fallback": "exportar Markdown da rodada"
}
```

Pedido:

```json
{
  "request_id": "r-2026-11-13-001",
  "capacidade": "pre_aprovar_ds",
  "entrada": {"figma_link": "https://www.figma.com/design/…?node-id=…", "marca": "Animale", "plataforma": "Web", "contexto": "proposta p-7f3a aceita"},
  "solicitado_por": "orquestrador",
  "confirmacao_humana": null
}
```

Resultado:

```json
{
  "request_id": "r-2026-11-13-001",
  "status": "concluido",
  "saida": {"parecer": "pre_aprovado_com_ressalvas", "bloqueios": [], "ressalvas": ["…"], "contrato_etapa": "stage.schema.json v…"},
  "artefatos": [{"tipo": "md", "caminho": "<raiz-escrita>/rodadas/r-…md"}],
  "origem_do_resultado": {"agente": "ds-gatekeeper", "versao": "0.2.13", "modelo": "…"},
  "custo": {"tokens_entrada": 0, "tokens_saida": 0, "leituras_figma": 0, "duracao_s": 0},
  "evidencia": ["…"]
}
```

Estado do conector: `desconhecido | disponivel | precisa_autorizar | cota_esgotada | bloqueado | erro`, com `mensagem` e `acao_corretiva` (E04).

## 5. Plano de spikes e riscos priorizados

Probabilidade (P) e impacto (I): A = alto, M = médio, B = baixo. Cards do Notion: números de E07–E18 **a confirmar** com o backlog (não reli o backlog nesta sessão); onde há card conhecido, ele está citado.

| # | Risco | P | I | Spike / mitigação | Card | Dono sugerido | Modelo |
|---|---|---|---|---|---|---|---|
| 1 | Proposta não volta sem rastro ou aceite ≠ 1 desfazer | M | A | Spike Gate 0 (ADR-004, critério dos 20 batches) | Gate 0 / propostas | Editor Engineer; Architect revisa | Sonnet codifica, Opus revisa |
| 2 | Canal local sem autenticação | A | A | Raízes confinadas já; token por conexão antes de 2º usuário (ADR-008) | Segurança (a confirmar) | Integration Lead | Sonnet |
| 3 | Fork não compila sem `craft-libs` / licenças não conferidas / CI do upstream não roda | M | A | Espelhar `craft-libs`, conferir licenças, CI próprio (ADR-005) | Fork/CI (a confirmar) | Integration Lead | Haiku (inventário), Sonnet |
| 4 | Snapshot de tokens vira segunda fonte ou fica velho | M | A | Cache fora do repo, hash + aviso de validade (ADR-002) | Adaptador de tokens | Code Bridge; DS Governance revisa | Sonnet, Opus revisa |
| 5 | Business Rule / Components tratados como estilo ou copy | M | M | Separação por grupo no adaptador; teste com a coleção inteira | Adaptador de tokens | Code Bridge | Sonnet |
| 6 | Cota do Figma MCP (200–600 leituras/dia) esgota | M | M | Uma leitura por coleção; cache; agendar retomada | Conectores | Connector Engineer | Haiku |
| 7 | Sem eventos: Claude não sabe o que o designer mudou | A (hoje) | A | Emitir `document.changed` (ADR-004, delta) | Delta/eventos | Editor Engineer | Sonnet |
| 8 | Origem por camada é falsificável / perdida | A | M | Tratar como informativa; não usar em governança (ADR-009) | Origem por camada | Architect | Opus |
| 9 | Conflitos de rebase com upstream em `cmd.rs`, `exec.rs`, `undo.rs`, `edit.rs` | A | M | Patches aditivos; `upstream-sync` semanal | Fork/CI | Integration Lead | Sonnet |
| 10 | Acoplamento a uma versão do ds-orchestrator | M | M | Registro por capacidade (ADR-006) | E16 (DS Experts) | Connector Engineer | Sonnet |
| 11 | Perfis de marca ausentes (só NV) | A | B | Modo "sem perfil" explícito (ADR-007) | E17 (Naty) | UX Interaction | Sonnet |
| 12 | Importação TSX promete mais do que entrega | M | M | Spike limitado com critério (ADR-003) | TSX (a confirmar) | Code Bridge | Sonnet |
| 13 | Direção de UX muda após teste do protótipo | M | M | Teste da dona com o protótipo; ADR-001 "sujeita a teste" | E06 / UX | UX Interaction | Opus |
| 14 | Lint de DS com regra inventada (escopos, nomes) | M | A | Regras só do Notion/Figma, citadas; o que não está escrito vira aviso | Lint de DS | DS Governance | Opus |
| 15 | Marca ArtCraft distribuída por engano | B | A | Rebranding antes de qualquer distribuição | Fork/CI | Integration Lead | Haiku |

Ordem dos spikes: **1 (Gate 0)** → 7 → 4/5 → 2 → 12.

## 6. Sequência proposta depois do Gate 0

Os nomes dos cards E07–E18 precisam ser conferidos no backlog; a sequência abaixo é por frente e indica onde cada card citado entra.

**G1 — Ciclo básico confiável (proposta + delta + tokens).**
- Frente A (engine, `edit.rs`/`undo.rs`/`cmd.rs`): propostas passo 0 → passo 1.
- Frente B (engine, só `exec.rs`): `document.changed` + delta.
- Frente C (fora do upstream): adaptador de tokens + snapshot + manifesto.
- Frente D (fora do código): CI próprio, espelho de `craft-libs`, conferência de licenças.
- **Paralelo:** A, C e D sim. B toca `exec.rs`, que A não toca no passo 0, então B pode rodar junto; no passo 1 de A, sequenciar.
- **Checkpoint humano G1:** demo do loop com cartão de proposta, aceitar/rejeitar e "o que mudou"; dona aprova antes de G2.

**G2 — Governança no canvas (origem, lint, Gatekeeper).**
- Frente E: origem por camada (depende de B).
- Frente F: lint de DS (crate `eureka-core`, leitura; regras do Notion).
- Frente G: contrato de conectores + Gatekeeper por capacidade (E16).
- Frente H: segurança do canal (token), se aprovado.
- **Paralelo:** F e G sim (arquivos diferentes). E depois de B. H isolado em `control_server.rs` ou proxy.
- **Checkpoint humano G2:** dona valida regras do lint com a liderança de DS; nenhuma regra nova sem fonte.

**G3 — Exploração e integrações.**
- Frente I: quadro de exploração (C) ligado ao gerador-propostas e perfis (E17).
- Frente J: export TSX + spike de importação limitada.
- Frente K: UI do app (painel de propostas, preview no canvas, selo do Gatekeeper) — crate L6.
- Frente L: rebranding.
- **Paralelo:** I e J sim. K toca `ui-egui`, isolar de L (`dialogs.rs`).
- **Checkpoint humano G3:** rodada completa do fluxo da Naty (problema → 3 a 5 propostas → escolha → crítica → Gatekeeper → refinamento), registro no Notion **só com confirmação**.

Regras para todas as fases: worktree por frente; nada de merge sem a dona; `cargo xtask ci` local antes de PR; sem atribuição de agente em PR para o upstream.

## 7. Decisões ainda pendentes da dona

| # | Decisão | Recomendação | Alternativas |
|---|---|---|---|
| D1 ✅ Aceita 13/11 | Segurança do canal local no MVP (ADR-008) | Raízes confinadas + porta aleatória + sem `online/live` já; token antes do 2º usuário | Aceitar risco só em máquina pessoal; proxy autenticado |
| D2 ✅ Aceita 13/11 | Onde fica o cache de tokens (ADR-002) | Fora do repo; no repo só manifesto sem valores; testes com fixture sintética | Versionar snapshot no repo; repo privado só de snapshots |
| D3 | Aliases no snapshot (ADR-002) | Guardar alias e valor resolvido | Só resolvido (como o exemplo atual) |
| D4 | Escopo `GAP` do Spacing usado em padding (ADR-002) | Aviso, não erro, até a liderança de DS decidir | Erro; ignorar escopos |
| D5 ✅ Aceita 13/11 | CI (ADR-005) | Workflow próprio do Eureka, sem mexer nos do upstream | Pedir acesso ao `craft-common-actions` |
| D6 | Nome e marca do produto (rebranding obrigatório) | Definir antes de G3 | — |
| D7 | Contribuir de volta ao upstream (ex.: token do canal, dry-run) | Sim, para mudanças genéricas; reduz conflito de rebase | Só consumir |
| D8 | Ponto de entrada do DS Experts (ADR-006) | Skills individuais por capacidade no MVP; `ds-workflow` para demandas longas | `ds-orchestrator` como entrada única |
| D9 | Critério para ampliar importação TSX (ADR-003) | Os 4 critérios do ADR-003 | Outro critério |

## 8. Conflitos encontrados entre relatórios e decisões

1. **HANDOFF S1-4 e o script do spike usam o `tokens.json` do artifact DS Animale** como entrada (`HANDOFF.md`, "Plano da Sprint 1" e "Como reproduzir"). A decisão 2 diz que a fonte é o Figma; o artifact é cópia. → Trocar a entrada do S1-4 e do `spike_hybrid_loop.py` pelo snapshot gerado do Figma.
2. **HANDOFF diz que os perfis de marca não existem.** E04 encontrou-os no `gerador-propostas`; a decisão 6 confirma. → HANDOFF desatualizado.
3. **"Customizações isoladas" × registro fixo de comandos.** Comandos novos precisam de 1 linha em `cmd.rs` (E01). Isolamento total não é possível; o mínimo é aditivo.
4. **Snapshot do exemplo com `aliasesResolved: true`** perde a cadeia de alias que o lint precisa (D3).
5. **Regra 15 do upstream × rodapé de atribuição** do fluxo do Claude Code: vale só para PRs ao upstream (E01).
6. **Origem "Claude (agent)" é auto-declarada** (E01), enquanto o benchmark (E03) trata origem por camada como diferencial. → Diferencial de UX, não de governança (ADR-009).
