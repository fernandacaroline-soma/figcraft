# E01 — Auditoria do fork FigCraft: arquitetura, licença, extensibilidade

Data: 13/11/2026. Branch `claude/eureka-design-lead-epvzr2`. Método: só leitura de código e docs, com `grep`.
**Nada foi compilado nem executado.** Não há Rust/cargo nesta máquina. Toda afirmação abaixo vem do texto do
código ou da documentação, com `arquivo:linha`. Onde é hipótese, está escrito "hipótese".

## Resumo

1. O FigCraft já entrega 80% da base do ciclo do Eureka: comandos únicos para tudo, `command.batch` atômico com um
   passo de desfazer, servidor MCP, canal de controle, variables com modos e importador de variables do Figma.
2. As lacunas do HANDOFF se confirmam: sem dry-run/proposta, sem eventos de edição, sem origem por camada,
   sem lint de DS, sem cliente LLM.
3. "Proposta sobre cópia" é **viável e barata**. O documento é persistente (`imbl`), o batch já faz rollback
   completo e já existe diff por nó. Estimativa: **M** (P para a versão mínima sem UI).
4. Licença é boa (MIT OU Apache-2.0). Os riscos reais são de **governança**: dependências git em `storytold/craft-libs`,
   CI compartilhado que não roda no fork e canal de controle **sem autenticação**.
5. Convivência com o upstream é possível se o Eureka entrar em crates novos e tocar poucos arquivos do upstream.

## Capacidades aproveitáveis

| Capacidade | Evidência | Serve ao Eureka como |
|---|---|---|
| Comando único para tudo (macro `spec!`) | `crates/engine/src/cmd.rs:57-61` (macro), `cmd.rs:73-132` (registro), `docs/parity-checklist.md:5` ("575 commands in the engine", gerado em 21/10) | Uma só API para Claude, menus e atalhos. O HANDOFF diz "~500": está subestimado |
| Parâmetro desconhecido é erro | `cmd.rs:176-186` (`refuse_unknown_params`) | Claude erra o nome de um parâmetro e recebe erro, não efeito silencioso |
| Catálogo de paridade aceita ids fora dele | `catalog.rs:308-321`; `command.batch`, `inspect.code`, `variable.import`, `codeConnect.set` não aparecem em `catalog.rs` e estão registrados | Comandos `proposal.*` podem existir sem mexer no catálogo |
| `command.batch` atômico, 1 desfazer, `$ref`/`$step` | `cmd/edit.rs:510-536`; `session/undo.rs:202-210` (`batch`), `undo.rs:264-267` (`grouped`), `automation/batch.rs:3-4,147-180` | Claude aplica uma proposta inteira ou nada. Passos referenciam o que o passo anterior criou |
| Batch com `onError: continue` | `edit.rs:512-518`; `undo.rs:213-232` (`batch_continue`) | Relatório por passo quando uma proposta é parcialmente inválida |
| Rollback completo de grupo | `undo.rs:322-352` (`group_checkpoint`), `undo.rs:360-387` (`restore_group_checkpoint`: documento, seleção, página, undo, redo, journal, eventos) | Base do dry-run (ver Spike) |
| Histórico por snapshots `imbl` | `crates/doc/src/lib.rs:201` (`nodes: imbl::OrdMap<NodeId, Arc<Node>>`); `Cargo.toml:72` (`imbl`); `doc_state.rs:310` (`Arc::make_mut`) | Clonar o documento é barato para os nós |
| Diff por nó já existe | `doc/src/lib.rs:416-428` (`changed_ids`, usa `imbl` diff, custo ~ tamanho da edição); `engine/src/derived.rs:4-11` (`changed_nodes`, varre tudo: mais lento) | Núcleo do `proposal.diff` |
| Servidor MCP, 19 ferramentas | `crates/mcp/src/tools.rs:95-115` (`TOOL_NAMES`). Nomes: `list_commands`, `run_command`, `batch`, `create_node`, `set_properties`, `inspect_document`, `get_node`, `select`, `export`, `screenshot`, `open_file`, `save_file`, `undo`, `redo`, `pointer`, `key`, `type_text`, `inspect_ui`, `wait_for_changes` (19 confirmadas) | Claude orquestra de fora. `run_command` chama qualquer comando novo sem mudar o MCP |
| MCP headless ou `--attach` | `docs/mcp.md:16-20`; `mcp/src/backend.rs:95-112` | Mesmo servidor para teste (headless) e para o app aberto |
| Canal de controle (JSON por linha) | `docs/control-protocol.md:1-30`; `ui-egui/src/control.rs:1462-1482` (`handle`, `engine.execute`); `apps/figcraft/src/control_server.rs:96-115` | Superfície que o script do spike já usa |
| Marcação "Claude (agent)" | `engine/src/session/exec.rs:55-66` (`execute_as`); `cmd/comment.rs:277-281` (`agent_author`); `mcp/src/server.rs:55-71` (nome vem de `clientInfo`); `sharing/agent.rs:1-5,12-14`; `ui-egui/src/live/agent.rs:13-20` | Avatar do agente no canvas. **Só isso**: ver correção abaixo |
| Eventos `events.poll` / `wait_for_changes` | `engine/src/events.rs:1-6,87`; `mcp/src/tools.rs:383,422-424` | Só **comentários** emitem. Ver Lacunas |
| Variables, modos, aliases, extensão de coleção | `cmd/variables_io.rs:257-330`; `doc/src/styles.rs:370-373` (`Variables`) | Modo por marca (Animale etc.) |
| `variable.import` lê o JSON de variables do Figma | `cmd/variables_io.rs:26-29` ("the local variables JSON … `variableCollections` and `variables` under `meta` or at the top level"); `format/src/rest.rs:115`; `format/src/rest/variables.rs:114,233` (`valuesByMode`, filtrado por modo) | **Confirmado.** Importa por nome: coleção igual entra no mesmo nome, modos e variáveis novos são acrescentados. Não lê DTCG nem Tokens Studio (busca por "DTCG"/"Tokens Studio" no engine: sem resultado) |
| Exportar variables | `cmd/variables_io.rs:21` (`json`, `css`, `tailwind`, `dart`) | Entrega tokens ao código sem cópia manual |
| Codegen | `engine/src/dev/codegen.rs:44,51` (`css|html|react|leptos|vue|tailwind|swiftui|compose|arkui|flutter`) | "Código sempre consistente". O HANDOFF não cita Leptos nem ArkUI |
| Code Connect | `dev/code_connect.rs:562,588,610,643` (`codeConnect.set/remove/export/import`); `apps/figcraft-cli/src/args.rs:45`, `files.rs:479` (`connect publish`, com `--dry-run`) | Liga componente a import/props da biblioteca de código |
| Plugins WASM sem rede | `docs/plugins.md:8-10` ("no file system, network, clock or randomness"); `crates/plugin/Cargo.toml:6` | **Confirmado.** Plugin não hospeda chat nem chama conector. (O app, fora do plugin, baixa diretórios de plugin: `docs/plugins.md:217`) |
| Build web (WASM) + host bridge | `docs/control-protocol.md:457-470` (`window.figcraft`, só com `?control`); `apps/figcraft-web/src/host.rs:6-8` (`?host=parent` via `postMessage`, só pai de mesma origem) | O Eureka pode viver num `<iframe>` de um host. Sem TCP na web |
| Import/export `.fig` | `docs/file-format-parity.md:10-13` (abre "best effort", escreve "experimentally"); `crates/format/src/fig.rs`, `fig/export.rs` | Sai do Figma por `.fig` ou por variables JSON. **Fidelidade não verificada** |
| Dados livres por camada | `doc/src/node.rs:552-553` (`plugin_data`); `cmd/props.rs:1108` (`pluginData` em `node.set`) | Gancho para marcar origem sem mudar o formato (ver Lacunas) |
| Modo "prévia de versão" | `doc_state.rs:95,263-286` (`start_preview`), `exec.rs:170-177` (`check_preview`: edição recusada durante a prévia) | Molde para o preview de proposta no canvas |

## Lacunas para o Eureka

| Lacuna | Evidência | Esforço |
|---|---|---|
| Proposta / dry-run de batch | Nenhum `dryRun` nos comandos (busca no engine e no MCP: só `--dry-run` do CLI `connect publish`, `files.rs:479-495`). `command.batch` aplica direto (`edit.rs:535`) | **M** (desenho no Spike) |
| Eventos de edição do documento | `events.rs:121` filtra `document.*`, mas nada emite: o único `push` de produção está em `session/exec.rs:29-51` e é chamado por `comment.*`; `document.changed` só aparece em `events_tests.rs:21,44`; a própria ferramenta diz "document events are not emitted yet" (`tools.rs:383`). **O HANDOFF está correto.** Há tudo para emitir: `exec.rs:69+` já calcula `before`/`after` | **P–M** (emitir `document.changed` com `changed_ids` ao fim de `execute`; cuidar de ruído em gestos de arrastar) |
| Origem por camada (IA × humano) | `Session.agent` existe só durante o comando (`lib.rs:160-162`, `exec.rs:59-65`); o resultado só vai para comentário e avatar. Presença some em 8 s e **não é salva** (`sharing/agent.rs:1-5,12`). Nada em `Node` registra autor (`doc/src/node.rs:449+`) | **M**. Atalho sem mexer no formato: gravar `pluginData["eureka.origin"]` no mesmo batch (`props.rs:1108`). Falta limpar a marca quando o humano edita (precisa do evento acima) |
| Lint de DS / proteção de componente oficial | Nenhum comando de lint (busca por `lint` nos `spec!`: sem resultado relevante). Instâncias só têm `check_instance_structure` (`exec.rs`, estrutura, não DS). Biblioteca importada é somente leitura (`variables_io.rs`: "never a library copy (read-only)"; `select_and_page`, `edit.rs:545+`) | **M–G**. Regras (token solto, hex fora de variável) são fáceis como leitura em `inspect`; bloquear antes de aplicar exige o hook de proposta |
| Cliente LLM no app | Nenhum cliente HTTP de modelo; rede só em `collab-net` (`xtask/src/layers.rs:30` restringe rede ao L6) | **G** e fora do MVP (decisão do HANDOFF: Claude fora do app) |
| Adaptador de tokens do DS | `variable.import` só lê JSON de variables do Figma (acima). O `tokens.json` do artifact "DS Animale Web" tem outro formato | **P–M** (conversor fora do app gera o JSON de variables; zero mudança no upstream) |

## Licença e dependências

- **Licença do código:** `MIT OR Apache-2.0` (`Cargo.toml:7`; `LICENSE-MIT:3`, `LICENSE-APACHE`, `NOTICE:1-5`). Permite o fork,
  inclusive uso comercial, com aviso de copyright mantido. Código novo do Eureka pode ter licença própria, desde que o
  `LICENSE-*` do upstream continue no repo.
- **Marcas ArtCraft não são livres:** `docs/brand/LICENSE-brand.txt:1-8` ("trademarks … NOT open source"; permissão limitada,
  revogável, "only" para o FigCraft). `NOTICE:5-7` repete. O nome, logo e mark ficam em `docs/brand/*` e são citados no
  app (`ui-egui/src/dialogs.rs:155-162`, links `getartcraft.com` e Discord) e em `apps/figcraft/build.rs:15`
  (`CompanyName = ArtCraft`). **O Eureka precisa trocar nome, logo, textos de "Sobre" e metadados do executável** antes de distribuir.
  Os `.md` das marcas não devem ser editados (`AGENTS.md`, regra de ativos).
- **Dependências git:** todas em `github.com/storytold/craft-libs`, três revisões fixas:
  `e11f9d9a…` (`craft-docpath`, `craft-window`, `craft-font-store`, `craft-font-catalog`, `craft-ui`, `craft-menu`, `craft-settings`;
  `Cargo.toml:34-58`), `e9829955…` (`craft-update`, `Cargo.toml:40`) e `fd5e2fe…` (`craft-web-api`, `craft-web-host` na web,
  `apps/figcraft-web/Cargo.toml:24-25`). Usadas em `crates/engine/Cargo.toml:24,39-40`, `crates/ui-egui/Cargo.toml:12,43`,
  `crates/text/Cargo.toml:31`, `apps/figcraft/Cargo.toml:22,24,40,64`.
  - Riscos: (a) o build depende de acesso ao GitHub; se o repo for privado ou sumir, o fork não compila sem espelho;
    (b) **licença dessas libs não verificada** (sem rede aqui); a lista de licenças permitidas do upstream está em
    `AGENTS.md:34-48`; (c) manter três revisões fixas dá trabalho a cada atualização;
    (d) `craft-update` é um atualizador: precisa ser desligado no Eureka, ou ele consultaria o canal de releases do upstream
    (hipótese; não li o crate).
- **Fontes** ficam fora do repo (`storytold/craft-fonts`, opcional; `AGENTS.md`, regra de fontes; `.cargo/config.toml:3`). O build funciona sem elas.
- **Regras do upstream que conflitam com o Eureka:**
  - *Rust only, sem JS/TS, sem webview* (`AGENTS.md:30`): **não conflita** com o MVP (Claude por MCP, UI em egui). Conflita se o
    Eureka quiser um painel de chat em webview ou front-end JS. O build web existente é preservado pela mesma regra.
  - *CI compartilhado* (`AGENTS.md:16,181-183`; `.github/workflows/ci.yml:22`, `house-rules.yml:26`): os workflows chamam
    `storytold/craft-common-actions@main`, repo que o fork pode não alcançar ou rodar. A regra proíbe mudar CI em PR de feature.
    **Sem CI, a única checagem é `cargo xtask ci` local** (não executado aqui).
  - *`house-rules.yml` observa `AGENTS.md` e `CLAUDE.md`* (`house-rules.yml:6-14`) e o bloco de regras tem hash
    (`AGENTS.md:1`): **editar esse bloco quebra o check.** Regras próprias do Eureka vão em arquivo separado
    (já é `docs/eureka/` e `.claude/`).
  - *Sem atribuição de agente em PR* (`AGENTS.md:20`, regra 15): conflita com o rodapé "Generated with Claude Code" que o fluxo do
    Eureka usa. Decisão da dona do produto: para PRs ao upstream, remover; no fork, é livre.
  - *Nunca crashar / sem `unwrap`* (`AGENTS.md:27,110-118`; `Cargo.toml:119-125`): vale também para código novo. Custo de disciplina, não de bloqueio.

## Segurança

- **Endereço:** o canal escuta só em `127.0.0.1:<porta>` (`apps/figcraft/src/control_server.rs:96-115`; `docs/control-protocol.md:9`).
  O `mcp --attach` conecta em `127.0.0.1:<porta>` (`mcp/src/backend.rs:95-112`; `docs/mcp.md:20`). Não é aberto para a rede.
- **Autenticação: nenhuma.** Busca por `token`/`auth` em `control.rs`, `control_server.rs` e `mcp/src` não acha verificação
  (os "owner token" achados são de arquivos publicados, `ui-egui/src/live.rs:57+`, outro assunto). Qualquer processo do mesmo
  computador que conheça a porta executa qualquer comando.
- **Limites existentes:** 8 conexões, fila de 8, linha de 32 MiB, timeout de 60 s (`docs/control-protocol.md:29-45`).
  Linha que não é JSON encerra a conexão, então uma página web não consegue usar o canal por HTTP (`control-protocol.md:24-27`).
- **O que um processo local malicioso poderia fazer** (leitura de `docs/control-protocol.md:94-111` e `docs/mcp.md:52-118`):
  ler e salvar arquivos pelo app (`file.open`, `file.save`, `file.export`); apagar ou alterar o documento; e, se o app tiver
  rede, publicar o documento em servidor (`online.*`, `live.*`) ou instalar plugin de diretório. O **confinamento é opt-in**:
  só vale se o app foi iniciado com `--automation-read-root/--automation-write-root`, e então bloqueia também a rede
  (`control-protocol.md:103-104`). O próprio doc admite a corrida "checa, depois abre" (`docs/mcp.md:115-118`).
  Sem os flags, o agente tem o mesmo alcance do usuário.
- **O nome do agente é declarado pelo cliente** (`mcp/src/server.rs:55-57`; `control.rs:1471`): qualquer cliente se diz
  "Claude" ou omite o campo e vira "usuário". Origem por camada baseada nisso é **informativa, não confiável**.
- **Validação de entrada:** varredura de lixo em todo comando em `crates/engine/src/tests.rs:393-495` (`every_command_survives_junk_params`,
  itera `all_commands()` na linha 481). Pula arquivo/importação/`file.revert` (linhas ~470-480), por isso esses comandos dependem do confinamento.
  `guard::run` converte pânico em erro e desfaz (`exec.rs:94`, `AGENTS.md:112-114`). **Todo comando novo entra no sweep automaticamente**
  e precisa de entradas de lixo próprias para seus parâmetros.
- **Recomendação:** no Eureka, iniciar o app sempre com roots confinados e adicionar um segredo na porta (token por conexão)
  antes de qualquer uso fora de teste. Isso toca `control_server.rs` (upstream) ou exige um proxy autenticado.

## Extensibilidade e convivência com o upstream

- **Camadas** (`xtask/src/layers.rs:8-26`): geom/color L0 → doc L1 → layout/text/pathops/collab/plugin L2 → render/format L3 →
  svg/pdf/vectordrawable L4 → engine L5 → ui-egui/mcp/collab-net L6 → apps.
  - Conversor de tokens, lint de DS, cálculo de diff e adaptador do Gatekeeper: **crate novo L4–L5** (depende de `doc`/`engine`;
    não pode usar egui nem rede: `layers.rs:30,46-55`).
  - Painel de propostas e preview: **crate novo L6** (ou módulo em `ui-egui`).
  - Conectores de rede (Gatekeeper, Notion): só L6, como `collab-net`.
  - **Atenção:** crate fora da `TABLE` é **ignorado** pela checagem (`layers.rs:43`, `continue`). Para valer, é preciso listá-lo em
    `layers.rs` (1 arquivo do upstream) ou aceitar que fica sem fiscalização.
  - `Cargo.toml:2` usa `crates/*`: crate novo entra no workspace sem editar o manifesto raiz. Deve ter `[lints] workspace = true`
    (`AGENTS.md`, regra de crates novos).
- **Comandos novos:** o registro é uma lista fixa dentro do engine (`cmd.rs:73-132`, uma linha `list.extend(...)` por módulo). Não há
  registro dinâmico: um crate L6 **não consegue** registrar comando no engine (L5 não depende de L6). Duas saídas:
  1. Comandos do Eureka dentro do engine, em arquivo novo (`cmd/proposal.rs`) + 1 linha em `cmd.rs` + `mod` em `cmd.rs:1-24`.
     O catálogo aceita (ver tabela). Mínimo de conflito, mas edita arquivos quentes do upstream.
  2. Lógica em crate Eureka e só um adaptador fino no engine.
- **Arquivos do upstream a tocar** (estimativa por leitura, não por build):

  | Entrega | Arquivos do upstream | Novos |
  |---|---|---|
  | Dry-run mínimo (`dryRun` em `command.batch`) | `cmd/edit.rs`, `session/undo.rs` (2) + 1 teste | 0–1 |
  | `proposal.*` com estado guardado | `lib.rs`, `session/new.rs` (campo no `Session`), `cmd.rs` (mod + registro) (3) | `cmd/proposal.rs`, testes |
  | Eventos `document` | `session/exec.rs` (1) | testes |
  | Origem por camada via `pluginData` | 0 (feito pelo orquestrador no batch) | script/regra |
  | Lint de DS (leitura) | `cmd.rs` (1) | crate ou módulo novo |
  | Crate L6 de UI | `xtask/src/layers.rs` (1), `ui-egui/src/lib.rs` (1) | crate novo |
  | Rebranding | `apps/figcraft/build.rs`, `ui-egui/src/dialogs.rs`, `README`, `docs/brand` (substituir) | assets novos + `ATTRIBUTION.md` |

  Total esperado: **4 a 8 arquivos do upstream** para o MVP, quase todos com mudanças aditivas (uma linha ou uma função).
- **Conflito esperado:** o upstream muda `cmd.rs`, `exec.rs`, `undo.rs` e `edit.rs` o tempo todo (o último merge do histórico é #2259).
  Manter os patches aditivos e pequenos, num branch `upstream-sync` rebaseado com frequência.

## 3 riscos críticos

1. **Canal de controle sem autenticação e confinamento opt-in.**
   Evidência: `control_server.rs:96-115`, `docs/control-protocol.md:94-111`, `docs/mcp.md:115-118`. Qualquer processo local comanda o app
   e lê/escreve arquivos; o nome "Claude (agent)" é auto-declarado (`mcp/src/server.rs:55-57`).
   Mitigação: iniciar sempre com roots de leitura/escrita; segredo por conexão; tratar a origem do agente como informativa;
   o Gatekeeper do DS não pode depender dela.
2. **Fork preso a `craft-libs` e a CI que não roda no fork.**
   Evidência: `Cargo.toml:34-58`, `.github/workflows/ci.yml:22`, `house-rules.yml:6-26`, `AGENTS.md:181-183`. Build precisa do GitHub, as
   licenças dessas libs não foram verificadas e não há CI próprio.
   Mitigação: espelhar `craft-libs` na organização, conferir licenças (lista em `AGENTS.md:34-48`), fixar as três revisões,
   rodar `cargo xtask ci` local em cada PR e abrir pedido ao dono do `craft-common-actions` ou criar CI próprio (decisão de governança).
3. **A promessa "aceitar = 1 desfazer / rejeitar sem rastro" ainda não existe e a base do desfazer tem exceções.**
   Evidência: o rollback de `grouped` restaura documento, seleção, histórico e journal (`undo.rs:360-387`), mas não restaura
   área de transferência, arquivos gravados, fontes baixadas nem sessões ao vivo; documento em sessão colaborativa usa outro
   histórico (`undo.rs:32,121`). Sem eventos (`tools.rs:383`), o Claude **não sabe o que o designer mudou** entre turnos além de reler tudo.
   Mitigação: o spike abaixo prova a base antes de a UI depender dela; proibir `file.*`, `online.*`, `live.*` e `plugin.*` dentro de
   proposta; emitir `document.changed` com `changed_ids`; desligar propostas em documento ao vivo no MVP.

## Spike recomendado — proposta sobre cópia

**Objetivo:** provar que dá para rodar um batch "no escuro", mostrar o diff, e depois aplicar como um único desfazer ou descartar sem rastro.

**Por que é viável (evidência):**
- `Document` é `Clone` e guarda os nós em `imbl::OrdMap<NodeId, Arc<Node>>` (`doc/src/lib.rs:193-201`): clonar compartilha os nós, custo
  quase constante. Fora do mapa há `Vec` (estilos, variables, comentários; `lib.rs:207-221`) e `images` (bytes em `Arc`, `lib.rs:166-170`): clonar
  é linear, mas pequeno para um DS (dezenas a centenas de tokens).
- O motor já guarda `Arc<Document>` antes de cada comando (`exec.rs:96`) e o histórico é uma pilha dessas cópias.
- `grouped` + erro forçado = rollback total (`undo.rs:264-304,360-387`), **inclusive o alocador de ids**, que fica no documento (`lib.rs:205`).
  Logo, rodar de novo gera os mesmos ids.
- Diff por nó: `changed_ids` (`lib.rs:416`). Diff por propriedade: `Node` é `Serialize` (`node.rs:449+`), então compara-se JSON de antes e depois por
  chave (ou usa a lista `NODE_KEYS` de `inspect.rs`, citada em `tools.rs:64`). Sem mudar o formato do arquivo.
- Preview no canvas: molde `start_preview` (`doc_state.rs:263`), que já mostra outro documento no lugar e recusa edição (`exec.rs:170-177`).

**Desenho mínimo (não implementado):**

| Comando | O que faz |
|---|---|
| `proposal.create {label, commands, base?}` | Dentro de `grouped`: roda o batch (`undo.rs:202`), captura `Arc<Document>` final, calcula diff, devolve `{id, touched: [ids], changes: [{id, name, props: {chave: [antes, depois]}}], variables, warnings}` e **força erro** para o rollback. Guarda `{id, commands, base_hash, touched}` (hash dos nós tocados antes) |
| `proposal.diff {id}` | Recalcula o diff contra o documento atual. Diz `stale: true` se o designer mexeu em nó tocado (`changed_ids(base, agora) ∩ touched`) |
| `proposal.accept {id}` | Se não estiver `stale`, roda **o mesmo batch de verdade** com o mesmo label: nasce **1 passo de desfazer** (`undo.rs:202`). Confere que o resultado tem os mesmos `touched` |
| `proposal.reject {id}` | Descarta o registro. Como `create` já fez rollback, não há rastro no documento |

Variante ainda mais enxuta (**passo 0**): só `dryRun: true` em `command.batch` e `ifUnchanged: hash` no aceite. O estado da proposta
fica no orquestrador (Claude), sem campo novo em `Session`. Cabe em `edit.rs` + `undo.rs`.

**Arquivos tocados:** ver tabela de extensibilidade (passo 0: 2 do upstream; versão com estado: 3 + 1 arquivo novo `cmd/proposal.rs`), `docs/mcp.md` e testes:
rollback sem rastro, aceitar = 1 passo (`undo_depth`, `undo.rs:30`), id estável, nó editado = `stale`, e entradas de lixo no sweep (`tests.rs:393`).

**Riscos do spike:**
1. Efeitos fora do documento não voltam no rollback (área de transferência, arquivos, fontes, sessão ao vivo). Mitigação: lista de comandos proibidos em proposta.
2. Rodar de novo pode divergir (fonte ausente, seleção/página diferente). Mitigação: comparar `touched`; senão `stale`.
3. O diff só vê valores finais. Para "por que mudou" é preciso o label por passo (já existe `as`/`name`, `edit.rs:524-528`).
4. Os 60 s do canal (`control-protocol.md:15-21`) limitam batches grandes; usar `timeoutMs`.
5. Sessão colaborativa usa outro histórico (`undo.rs:32`): fora do MVP.
6. Hipótese de desempenho (não medida): documentos grandes com muitos comentários/estilos. Medir no spike.

**Estimativa:** passo 0 (dryRun + guarda de base + testes) **P** (2–4 dias). `proposal.*` completo sem UI **M** (1–2 semanas). Preview no canvas e painel de
aceitar/rejeitar na UI: **M–G**, em outro card.
**Critério de sucesso:** em 20 batches reais do Animale, `create` → `reject` deixa `Document` igual (`==` do `PartialEq` em `lib.rs:192`) e `undo_depth` igual; `create` → `accept` soma exatamente 1 ao `undo_depth`.

## Demo do estado atual (roteiro; não executado)

Não rodei nada. O roteiro abaixo é o do HANDOFF (`docs/eureka/HANDOFF.md`, "Como reproduzir o teste"), conferido contra o script
`docs/eureka/spike_hybrid_loop.py`. Quem tiver Rust 1.95 (`Cargo.toml:9`) faz assim:

1. `cargo build -p figcraft` (primeira vez baixa as dependências git; precisa de internet).
2. Escolha uma porta livre e grave em `/tmp/eureka/port`; suba o app: `cargo run -p figcraft -- --control <porta>`.
3. Tenha o `tokens.json` do artifact "DS Animale Web" (não está no repo).
4. Rode, em ordem: `generate`, `manual`, `read`, `refine`:
   `python3 docs/eureka/spike_hybrid_loop.py /tmp/eureka <passo> <caminho>/tokens.json`.
5. **O que o roteiro deve mostrar** (segundo o HANDOFF, sem eu ter reproduzido): variables do Animale importadas; card gerado vinculado a tokens;
   edição manual no app (texto e uma cor `#7A4B2A` fora do DS); Claude lê a cor solta; devolve aos tokens; um ⌘Z desfaz o refino inteiro.
6. **O que o roteiro não mostra:** proposta antes de aplicar, aviso do que mudou, origem por camada, lint de DS.

Para inspeção sem janela: `figcraft-cli mcp` (headless) e `list_commands` com `filter: "proposal"` deve voltar vazio (hoje não existe nada com esse nome).

## O que não foi verificado

- Nada foi compilado, testado nem executado: sem `cargo`, sem rodar o app, sem rodar o script, sem `cargo xtask ci/layers/parity`.
- Contagem exata de comandos: o `docs/parity-checklist.md` diz 575, mas é de 21/10; a busca por `spec!(` dá cerca de 640 ocorrências. Só `figcraft-cli commands` dá o número real.
- Licenças de `craft-libs` e de todas as dependências do `Cargo.lock`; se `storytold/craft-libs` e `craft-common-actions` são acessíveis ao fork.
- Comportamento de `craft-update` no Eureka (hipótese de que consulta releases do upstream).
- Se o rollback de `grouped` cobre todos os efeitos de comandos específicos (li o mecanismo, não rodei casos).
- Desempenho de clonar/diff em documentos reais grandes; tamanho do `Vec` de histórico (limite 1000, `doc_state.rs:159`) ao guardar propostas.
- Fidelidade de importação/exportação `.fig` com arquivos reais; a própria doc diz "não checado" (`file-format-parity.md:10-13`).
- Código de `crates/ui-egui` para o preview: só li `live/agent.rs`; o custo de UI é estimativa.
- Números de linha são desta branch em 13/11/2026; o upstream muda rápido.

## Correções ao HANDOFF

- "~500 comandos": o contador do repo diz 575 (21/10) e há ~640 `spec!(` hoje. Não é erro grave.
- "O app mostra o agente com avatar e contorno": é presença visual temporária de 8 s, não é salva e não marca camadas (`sharing/agent.rs:1-5,12`).
  A "marcação de agente" real é a assinatura de comentários.
- "Sem rede nos plugins": correto para o plugin; o app tem rede própria (diretório de plugins, `collab-net`, `online.*`).
- Não achei afirmação errada sobre `variable.import`, eventos, lacunas ou as 19 ferramentas: todas confirmadas.
