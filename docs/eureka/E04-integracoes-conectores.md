# E04 — Mapa de integrações e conectores do Eureka Design

Data: 13/11/2026. Card Notion E04. Autor: Connector Engineer. Somente leitura em Notion, Figma e GitHub. Nada foi instalado, publicado ou gravado fora deste arquivo.

Legenda: **[V]** verificado (fonte ou teste citados) · **[I]** inferido · **[NV]** não verificado.

## Resumo

1. O Claude dirige o Eureka pelo servidor MCP do FigCraft (stdio). Funciona no Claude Code, no Claude Desktop e no Agent SDK. **[V]** (`docs/mcp.md:1-20`).
2. **Não existe API pública para ler ou escrever Artifacts a partir do app.** Existe a ferramenta `Artifact` dentro do Claude Code/Desktop logado, e a Compliance API (admin: listar, baixar versão, excluir). O caminho é arquivo: exportar do Eureka, publicar pelo Claude Code. **[V]**
3. Tokens Figma → Eureka: o Claude lê as variables pelo MCP remoto do Figma, grava um JSON no formato `meta.variables` e chama `variable.import` por caminho. Sem cópia manual. REST de variables exige Enterprise. **[V]** (leitura via MCP e conversão: **[I]**, não testei o Figma MCP: ele pede autorização nesta sessão).
4. Os agentes da Naty (`gerador-propostas`) e o DS Experts Studio (`ds-orchestrator`) são plugins do Claude Code. O orquestrador chama as skills na sessão principal. Subagentes não chamam outros subagentes. **[V]**
5. O "007" é o agente de Tecnologia (Jira → código → PR por marca). Não há repositório visível no GitHub da org. Vira pergunta para a dona.

## Matriz

| Sistema | Interface | Autenticação | Leitura / escrita | Onde executa | Limites | Evidência | Fallback |
|---|---|---|---|---|---|---|---|
| **FigCraft/Eureka (alvo)** | MCP stdio (`figcraft-cli mcp --attach PORT`), 19 ferramentas; canal de controle TCP | Nenhuma documentada; só escuta em `127.0.0.1` **[V]**. Sem token **[I]** | L e E (editar, inspecionar, exportar, screenshot, `run_command`, `batch`) | Máquina local; app aberto (attach) ou headless | 60 s por pedido (`--timeout` 1–600); 32 MiB por linha de entrada; 512 MiB por resposta; `--attach` não aceita raízes de arquivo (só o app, com `--automation-read-root/--automation-write-root`) | `docs/mcp.md:7-60`, `docs/control-protocol.md:3-12`, `crates/mcp/src/tools.rs:153-376` | Headless (`figcraft-cli mcp`), sem janela |
| **Claude Code (cliente MCP)** | stdio, HTTP (recomendado), SSE (obsoleto), WebSocket por JSON | OAuth (`/mcp`, `claude mcp login`), header Bearer, env | Cliente: chama ferramentas; permissões por ferramenta | Terminal local ou Desktop | Escopos `local`/`project`/`user`; `.mcp.json` entra em git se escopo `project` | https://code.claude.com/docs/en/mcp | Claude Desktop |
| **Claude Desktop** | stdio local via `claude_desktop_config.json` (`~/Library/Application Support/Claude/`) | Aprovação por ação na UI | Cliente | App local | Precisa reiniciar para ler config; Artifacts exigem versão ≥ 1.13576.0 | https://modelcontextprotocol.io/docs/develop/connect-local-servers; https://code.claude.com/docs/en/artifacts | Claude Code |
| **Claude Agent SDK** | `mcpServers` em código (stdio/http/sse) ou `.mcp.json`; permissão por `allowedTools` (`mcp__figcraft__*`) | OAuth não é interativo: token em `headers` ou login prévio no Claude Code | Cliente programático | Processo próprio do orquestrador | Servidor leva até 30 s para conectar (`MCP_TIMEOUT`); saída de ferramenta > 25 mil tokens vira arquivo | https://code.claude.com/docs/en/agent-sdk/mcp | Claude Code interativo |
| **Claude API (Messages)** | HTTP; MCP via cliente próprio **[NV]** | Chave de API | — | Nuvem | Não pesquisado a fundo; o caminho do MVP é Claude Code/SDK | — | SDK |
| **Claude Artifacts** | Sem API pública de escrita. Dentro do Claude Code/Desktop: ferramenta `Artifact` (publicar, ler, listar, excluir). Compliance API: `GET /v1/compliance/apps/code/artifacts`, `GET .../{id}/versions/{version_id}`, `DELETE .../{id}` | Sessão logada no claude.ai (ferramenta); chave de Compliance (API) | Ferramenta: L e E na conta da pessoa. Compliance: L e excluir, nunca criar/editar | Ferramenta: máquina local. Compliance: nuvem Anthropic | Ferramenta indisponível no Agent SDK, em `-p`, com chave de API, Bedrock/Vertex/Foundry, ZDR/HIPAA/CMEK. Arquivo `.html`/`.htm`/`.md`, até 16 MiB, uma página, CSP sem fetch externo | https://code.claude.com/docs/en/artifacts; https://platform.claude.com/docs/en/api/compliance/code/artifacts/list | **Arquivo**: Eureka exporta HTML/PNG/SVG e `tokens.json`; o Claude Code publica. Volta: Claude Code lê o artifact e grava o arquivo local |
| **Figma remoto (MCP)** | MCP HTTP `https://mcp.figma.com/mcp` (URL no Kit de design do Notion). Leitura: `get_design_context`, `get_variable_defs`, `search_design_system`, `get_metadata`, `get_screenshot`. Escrita: `use_figma`, `generate_figma_design`, `create_new_file`, `add_code_connect_map` | OAuth (nesta sessão ele pede autorização via `/mcp`) | L e E. Escrever fora do rascunho exige assento Full | Nuvem Figma | **Cota de leitura por dia/min**: assento Full/Dev em Enterprise 600/dia, 20/min; Organization 200/dia, 15/min; Professional 200/dia, 10/min; View/Collab 20/mês. Ferramentas de escrita citadas estão fora da cota. Escrita gratuita no beta, depois por uso | https://developers.figma.com/docs/figma-mcp-server/rate-limits-access/; https://developers.figma.com/docs/figma-mcp-server/ | Export manual do JSON de variables |
| **Figma REST (variables)** | `GET` variables locais e publicadas; `POST` variables | Token com escopo `file_variables:read` / `file_variables:write` | GET: qualquer membro; POST: assento Full/admin | Nuvem Figma | **Exige assento Full em organização Enterprise**; convidado não usa | https://developers.figma.com/docs/rest-api/variables/ | MCP remoto (linha acima) |
| **Formato de tokens do Eureka** | Comando `variable.import {variables}`: objeto, texto JSON ou caminho (até 256 MiB). Lê `variableCollections` e `variables` em `meta` ou no topo, com `valuesByMode`, `resolvedType`, `variableCollectionId`, aliases | Nenhuma | E | Local, dentro das raízes de escrita/leitura | **Não lê DTCG nem Tokens Studio** (HANDOFF). Une coleção de mesmo nome, casa modos por nome, pula tipo diferente com aviso | `docs/mcp.md:1255-1275`, `crates/format/src/rest/variables.rs:215-240`, `crates/engine/src/variables_io_tests.rs:176-190`, `docs/eureka/HANDOFF.md` | `variable.export` para o caminho inverso (Eureka → JSON) |
| **Notion (MCP)** | MCP HTTP `https://mcp.notion.com/mcp` (URL no Kit de design). Ferramentas vistas aqui: `notion-search`, `notion-fetch`, `notion-create-pages`, `notion-update-page`, `notion-delete-pages`, `notion-create-comment`… | OAuth da pessoa | L e E (criar, editar, excluir) | Nuvem Notion | Escrita só com confirmação humana no Eureka. Cota e endpoint não confirmados na doc oficial (a página de visão geral não traz) **[NV]** | Lista de ferramentas desta sessão; https://developers.notion.com/docs/mcp; página "Kit de design" (Notion) | Exportar a rodada como Markdown e a dona cola |
| **DS Experts Studio / ds-orchestrator** | Plugin do Claude Code. Skills `/ds-orchestrator:ds-*`; agentes pelo `Agent` com `subagent_type: ds-orchestrator:ds-gatekeeper` etc. Contrato JSON `stage.schema.json` | Instalação do marketplace privado (`claude plugin marketplace add Soma-UXDesign/ds-orchestrator`); Figma e Notion pelos conectores da sessão | Audit, Architect e Gatekeeper só leem (`Read, Glob, Grep`). Builder, Flows e Documentação usam MCP da sessão e podem escrever | Sessão do Claude Code do usuário | Agentes restritos não chamam outros agentes. Teto de 95% de uso. Idioma pt-BR. Merge/publicar exigem liderança | `ds-orchestrator/README.md`, `plugins/ds-orchestrator/agents/ds-gatekeeper.md:1-30`, `plugins/ds-orchestrator/contracts/stage.schema.json` | Chamar o Gatekeeper direto pela skill, sem Workflow |
| **Agente da Naty (provável: `gerador-propostas`)** | Plugin do Claude Code: `/gerador-propostas:gerador-propostas <marca> <plataforma> <hipótese>` | Marketplace privado `Soma-UXDesign/gerador-propostas`; conectores Notion, Figma e navegador | Gera 3 a 5 propostas; escreve Notion e Figma só com confirmação | Sessão do Claude Code | Um agente para todas as marcas; perfil de marca em `references/marcas/`; sem perfil, só usabilidade + DS | https://github.com/Soma-UXDesign/gerador-propostas (README, `SKILL.md`, `references/formato-saida.md`) | Rodar a skill no Claude Code e importar o resultado (Markdown + links Figma) |
| **007** | Agente Claude Code (`agent_007`, skill `/agent-007`) que leva uma task do Jira ao código e ao PR, por marca | Não verificado | Lê Jira e Notion; escreve git/Jira **[NV]** | Instalação local do dev (`.build/`) | Gates com o dev na sessão principal; vários repos | Notion: "Operation 007…" (VERTAPP-5663) e "Melhorias no 007" (VERTAPP-5900) | Sem integração no MVP |
| **`@soma-monorepo/core-components`** | Código (React/TS/SCSS). Amostra local, sem hash de commit | Repositório privado **[NV]** | Só leitura da amostra | Local | Sem acesso ao repositório oficial (HANDOFF). Spinner, ícones e tipografia fora da amostra | `teste-codigo-button/RELATORIO.md`, `button.code.json:9-10` | Code Connect por arquivo `figcraft.connect.json` |
| **Code Connect do FigCraft** | `codeConnect.set/import/export`; `figcraft-cli connect publish <file.figcraft> [--mappings FILE] [--dry-run]` | Nenhuma | E no arquivo de design (salva inteiro ou nada) | Local ou CI | Casa por **nome de componente**; até 5.000 links e 4 MB; templates de 4.000 caracteres; sem execução do template | `docs/cli.md:299-315`, `docs/mcp.md:1765-1795` | Editar na aba Inspect |
| **Maze, Crazy Egg, tracking** | **[NV]** | **[NV]** | **[NV]** | **[NV]** | Não pesquisado (escopo do card: uma linha) | — | Importar relatórios por arquivo |

## Respostas diretas

### Artifacts

- Veredito: **sem integração direta**. Proof: a doc do Claude Code descreve a ferramenta de sessão e a Compliance API como únicas superfícies programáticas. A Compliance API lista, baixa versão e exclui; não cria nem edita. **[V]**
- A ferramenta de sessão funciona só no Claude Code CLI ou Desktop logado com conta claude.ai. No Agent SDK e em `-p` fica desligada por padrão. Logo, um orquestrador feito com o SDK **não** publica artifact por ela. **[V]**
- Fallback de exportação: Eureka grava HTML (`inspect.code`, linguagens HTML/React/Vue, `docs/mcp.md:383`), PNG (`export`, `screenshot`) e `tokens.json` (`variable.export`) numa raiz de escrita. Um Claude Code interativo publica o arquivo.
- Fallback de importação: o Claude Code lê o artifact (resumo + código-fonte salvo local) e entrega um arquivo. Importar HTML/React como camadas editáveis no Eureka **não foi verificado** **[NV]**. Tokens em `project/tokens.json` do artifact "DS Animale Web" são importáveis só depois de convertidos para o formato `meta.variables`. **[I]**

### Tokens: caminho mais curto

1. Claude lê as variables do arquivo Figma pelo MCP remoto (`use_figma` somente leitura, ou `get_variable_defs`). A equipe já fez essa leitura via Plugin API em 27/10 (`teste-codigo-button/tokens-figma-button.json`). **[V]** `get_variable_defs` devolve só a seleção **[I]**, então a coleção inteira pede `use_figma`.
2. Claude monta `{"meta": {"variableCollections": …, "variables": …}}` com `valuesByMode`, `resolvedType` e `variableCollectionId`, e grava na raiz de escrita.
3. Claude chama `run_command variable.import {variables: "<caminho>"}`. Resposta: `{collections, variables, updated, warnings}`. Reimportar atualiza no lugar. **[V]**
4. Regra de nome do DS: minúsculas, `/` e espaço viram `-` (HANDOFF). Conferir `warnings` antes de aceitar.
- Cota: cada leitura entra na cota do Figma (200 a 600 por dia por assento). Uma leitura da coleção Tokens WEB cobre as 12 marcas.
- Não usar REST se a conta não for Enterprise com assento Full. Pergunta 3.

### Como acionar o ds-orchestrator

- No Claude Code: `/ds-orchestrator:ds-gatekeeper <link Figma com node-id>` ou, no código, `Agent` com `subagent_type: "ds-orchestrator:ds-gatekeeper"`. Entrada: contexto estruturado do coordenador (link, snapshots Figma/Notion). Saída: contrato JSON da etapa (`stage.schema.json`) ou `read_requests` e `blockers`. **[V]**
- O coordenador precisa ser a **sessão principal**, porque os agentes proíbem chamar outros agentes (`ds-gatekeeper.md`: "Não chamar outro agente"). Se essa sessão também tiver o servidor `figcraft` configurado, ela aciona os dois. **[V/I]**
- No Agent SDK, carregar plugins locais por opção do SDK existe na doc (página de plugins do Agent SDK), mas **não testei** **[NV]**.
- Diferença de nome: **DS Experts Studio** = `plugins/ds-experts-studio`, as seis especialidades independentes, sem `ds-workflow`. **ds-orchestrator** = as seis + `ds-workflow`. Mesma fonte (`plugins/ds-orchestrator`), sincronizada por `scripts/sync_ds_skills.py`. `ds-skills` é o nome antigo. A pasta tem ainda `ds-ux-partner` (Builder Criativo) em `agents/`, que o README não lista entre as seis. **[V]** (`README.md`). No Notion, "DS Experts Studio" aparece como nome do conjunto (cards E16 e Estratégia). **[V]**

### Naty e 007

- **Naty**: não achei uma página com o nome "Naty" no Notion. Achei o card E17 ("Integrar agente de exploração da Naty") e, no GitHub da org, o repositório privado `Soma-UXDesign/gerador-propostas`: "Plugin do Claude Code com o Gerador de Propostas de UX (ideação, antes da validação no DS Gatekeeper)". Os commits são de `natalysilva-star` (última versão 0.3.0, 13/11/2026). **A ligação "Naty = natalysilva-star = gerador-propostas" é inferida.** O fluxo do repositório bate com o do HANDOFF (problema → 3 a 5 propostas → escolha → crítica → Gatekeeper) e traz `references/marcas/_modelo.md` e `nv.md`. Ou seja, os perfis de marca que o HANDOFF dizia não existir existem **nesse repositório**, não no figcraft. **[V]**
  - Entrada: `<marca> <plataforma> <hipótese ou problema>`. Saída: Markdown no formato de `formato-saida.md` (propostas A–E, comparativo, preview do registro Notion, uma pergunta) + links Figma com node-id na branch. Estados: `PRE_VOO … CONCLUIDO`. Roda dentro do Claude Code, não como serviço.
  - Também existe `Soma-UXDesign/animale-ux-partner` (UX Partner da Animale), fora do escopo.
- **007**: página "Operation 007: orquestração em dois níveis, fluxo em código…" (Notion, status Concluído, Jira VERTAPP-5663) e "Melhorias no 007" (VERTAPP-5900). É o fluxo de Tecnologia que leva a task do Jira ao código, passando por fases, especialistas das libs e gates com o dev; hoje roda em duas camadas (agente + fluxo em código). Não achei repositório no GitHub da org nem na conta `fernandacaroline-soma` (busca por `agent_007`, `agent-007`, `007` sem resultado; pode estar em outra org ou privado sem acesso). **[V]** para o que é; **[NV]** para onde está o código.

## Arquitetura de conexão recomendada para o MVP

Quem chama quem:

```
Designer ──(UI do Eureka: pede ação, vê estado, decide)──► Eureka app
                                                              ▲   │ canal de controle 127.0.0.1
                                                              │   ▼
Orquestrador: Claude Code (sessão principal, logada) ── MCP stdio ── figcraft-cli mcp --attach PORT
   │   plugins: ds-orchestrator (Gatekeeper, Builder…), gerador-propostas (Naty)
   ├── MCP HTTP ──► Figma remoto (ler tokens/componentes; escrever só com confirmação)
   ├── MCP HTTP ──► Notion (ler; escrever só com confirmação)
   └── ferramenta Artifact ──► claude.ai (publicar/ler, arquivo → página)
```

- O Eureka **não** fala com Notion, Figma nem Artifacts. Ele aciona o orquestrador, mostra estado e resultado e coleta a decisão.
- Os agentes externos continuam independentes. O orquestrador só chama as skills deles e repassa o resultado ao Eureka.
- Troca de dados por arquivo numa raiz de escrita (`tokens.json`, HTML, PNG, `figcraft.connect.json`), nunca por cola manual.
- Configuração: `claude mcp add --transport stdio figcraft -- figcraft-cli mcp --attach 7979` em escopo `user` (o Kit de design da Estilo já exige `--scope user` para não criar `.mcp.json` em repositório). App: `figcraft --control 7979 --automation-read-root DIR --automation-write-root DIR`.
- Confirmação humana obrigatória: criar/editar/excluir no Notion, escrever/excluir no Figma, publicar ou compartilhar artifact, `connect publish` (grava no arquivo), merge. Aplicar com `allowedTools` só nas leituras; as escritas ficam sem permissão prévia. No SDK não há prompt interativo, então o orquestrador precisa de um gancho de aprovação **[I]**.
- Se o orquestrador virar um programa (SDK), ele perde a ferramenta Artifact. Manter o Claude Code interativo para publicar.

## Rascunho: campos de um contrato de conector

Rascunho, não implementar.

| Campo | Descrição |
|---|---|
| `id`, `nome`, `versao_contrato` | Identificador estável, nome exibido, versão do contrato |
| `capacidade` | Lista de verbos: `ler_tokens`, `propor_alternativas`, `pre_aprovar_ds`, `publicar_pagina`… |
| `interface` | `mcp_stdio` \| `mcp_http` \| `cli` \| `arquivo` \| `skill` |
| `entradas` | Schema tipado por capacidade (tipo, obrigatório, exemplo, limite de tamanho) |
| `saidas` | Schema tipado + `artefatos` (arquivo, formato, caminho dentro da raiz de escrita) |
| `permissoes` | `leitura`, `escrita`, `rede`, `raizes_de_arquivo`, `contas_necessarias` |
| `acoes_com_confirmacao` | Lista de ações que exigem decisão humana, com o texto mostrado à pessoa e o que será alterado |
| `idempotencia` | Se repetir é seguro; `request_id` para evitar duplicar (o attach não reenvia em falha de transporte) |
| `status` | `desconhecido` \| `disponivel` \| `precisa_autorizar` \| `cota_esgotada` \| `bloqueado` \| `erro`, com mensagem e ação corretiva |
| `autenticacao` | Tipo (OAuth, header, login do claude.ai), quem autoriza, validade |
| `custo` | Tokens estimados e medidos, chamadas contra cota (ex.: leituras do Figma por dia), tempo |
| `limites` | Timeout, tamanho máximo, taxa, concorrência |
| `origem_do_resultado` | Quem produziu (agente, versão, modelo) para marcar "IA" por camada |
| `fallback` | Caminho por arquivo quando o conector não está disponível |
| `evidencia` | Link da doc ou teste que prova a capacidade e data da verificação |

## Bloqueios e perguntas para a dona

1. **Naty**: confirma que o "agente de exploração da Naty" é o plugin `Soma-UXDesign/gerador-propostas` (autoria `natalysilva-star`)? Se não, qual é o repositório?
2. **007**: qual o repositório e quem mantém o `agent_007`? O Eureka deve acioná-lo (por exemplo, entregar o handoff de código) ou só ignorar no MVP?
3. **Figma**: a organização está em plano Enterprise com assento Full para você? Isso decide se a REST de variables é opção. Sem ela, o caminho é o MCP remoto, com cota diária.
4. **Segurança local**: o canal de controle do FigCraft não tem autenticação documentada, só escuta em `127.0.0.1`. Aceita esse risco no MVP (qualquer programa da máquina pode editar o arquivo aberto) ou exige um token?
5. **Artifacts**: o fluxo pode depender de um Claude Code interativo para publicar (o SDK não publica)? Ou prefere só exportar arquivo e publicar à mão?
6. **Biblioteca de código**: Tecnologia pode liberar leitura do `@soma-monorepo/core-components` (com hash de commit)? Hoje só há a amostra de 27/10.
7. **Aprovação nas escritas**: ok usar `allowedTools` só com leituras e deixar as escritas pedirem confirmação? Isso é decisão de governança.

## O que não foi verificado

- Conexão real do Figma MCP: nesta sessão ele pede autorização. Cota consumida, formato exato de `get_variable_defs` e leitura da coleção inteira por `use_figma` **[NV]**.
- Execução real de `figcraft-cli mcp --attach` com Claude Code ou Desktop (li a doc e o código, não rodei o app).
- Endpoint, escopos e cota do Notion MCP na doc oficial (a visão geral não os traz; a URL vem do Kit de design do time).
- Se o Agent SDK carrega os plugins `ds-orchestrator` e `gerador-propostas`.
- Importar HTML/React publicado em artifact como camadas no Eureka.
- Contrato de entrada/saída do 007 e onde está o código.
- Autoria real de "Naty" (inferida pelo autor dos commits).
- Maze, Crazy Egg e tracking: nenhuma pesquisa.
- Claude API (Messages) como cliente MCP direto.
- Se o plano da conta Figma da equipe é Enterprise.
- Pesquisa no GitHub limitou-se à org `Soma-UXDesign` (4 repositórios visíveis) e à conta `fernandacaroline-soma` (1 repositório); outras orgs não consultadas.

## Fontes

- FigCraft: `docs/mcp.md`, `docs/control-protocol.md`, `docs/cli.md`, `crates/mcp/src/tools.rs`, `crates/format/src/rest/variables.rs`, `docs/eureka/HANDOFF.md`.
- ds-orchestrator: `README.md`, `plugins/ds-orchestrator/agents/ds-gatekeeper.md`, `docs/transfer-to-soma-uxdesign.md`.
- gerador-propostas: https://github.com/Soma-UXDesign/gerador-propostas (leitura via `gh api`).
- Claude: https://code.claude.com/docs/en/mcp · https://code.claude.com/docs/en/agent-sdk/mcp · https://code.claude.com/docs/en/artifacts · https://platform.claude.com/docs/en/api/compliance/code/artifacts/list · https://modelcontextprotocol.io/docs/develop/connect-local-servers
- Figma: https://developers.figma.com/docs/rest-api/variables/ · https://developers.figma.com/docs/figma-mcp-server/ · https://developers.figma.com/docs/figma-mcp-server/rate-limits-access/
- Notion: https://developers.notion.com/docs/mcp; páginas E04, E17, "Operation 007", "Melhorias no 007", "Kit de design · Skills, Agentes & Conectores".
- Teste de código: `/Users/fernanda.caroline/Claude/teste-codigo-button/RELATORIO.md`, `button.code.json`, `tokens-figma-button.json`.
