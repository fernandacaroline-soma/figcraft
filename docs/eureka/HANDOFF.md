# Eureka Design — handoff da Sprint 0 (checkpoint 1)

Data: 13/11/2026. Branch: `claude/eureka-design-lead-epvzr2`. Página de revisão (privada, da dona do
projeto): https://claude.ai/artifact/M963EoB2qedTkWvXz9gR7S

## Papel e regras do projeto

- O agente atua como **Product Engineering Lead e Orchestrator** do Eureka Design: um fork do
  FigCraft que vira um workspace híbrido. O Claude gera, o designer edita visualmente, o Claude entende o
  estado editado e continua refinando, com o código sempre consistente.
- A dona do produto decide UX, arquitetura fundamental, regras de DS, publicações e ações
  externas, exclusões, merges e passagem de gates. **Nunca fazer merge, nunca registrar no Notion nem
  excluir no Figma sem confirmação.**
- O Design System de Design é a fonte da verdade. A biblioteca de código de Tecnologia serve só
  para compatibilidade. Nada de biblioteca paralela mantida à mão. Não alterar tokens, componentes ou
  regras globais.
- Fluxo de exploração (Naty): problema/hipótese → 3 a 5 propostas com evidências → escolha humana →
  crítica de Design → DS Gatekeeper → refinamento. O Notion guarda tudo; o Figma recebe só a escolhida.
- Perfis de marca: `references/marcas/` derivados de `_modelo.md`. **Não existem neste repo.** Sem
  perfil, trabalhar só com usabilidade e DS.
- Subagentes: Haiku para tarefas mecânicas, Sonnet para implementação e pesquisa, Opus para
  arquitetura, governança e UX críticas. Registrar consumo quando houver dado.
- Cada checkpoint mostra: o que foi feito, uma demonstração visual, o que funciona e o que não funciona, os testes e
  riscos, as decisões pendentes e o próximo passo.
- Seguir também `AGENTS.md` (regras do FigCraft: tudo é comando, nunca crashar, layering, `cargo xtask ci`).
- Usar a skill `.claude/skills/eureka-ux-ai/SKILL.md` em qualquer trabalho de UX.

## Fontes (links)

- Estratégia e roadmap (Notion): https://app.notion.com/p/417690e9a7e281479055d61838d94aa0
- Backlog (Notion): https://app.notion.com/p/5f17912cbab64596a257ae6fddabf514
- **Ainda não lidos.** O conector do Notion falhou na sessão em nuvem. Nenhum card (E01–E04) foi
  dado como concluído. Primeira tarefa da próxima sessão: ler a estratégia, o backlog e os cards
  E01–E04, e alinhar este plano a eles.

## O que foi feito

1. **Auditoria do FigCraft** (Sonnet, verificada por amostragem no código):
   - ~500 comandos; menus, CLI, plugins e agentes chamam os mesmos ids.
   - Servidor MCP com 19 ferramentas (`crates/mcp/src/tools.rs`), em modo headless ou `--attach PORT`
     no app aberto. O app mostra o agente com avatar e contorno "Claude (agent)".
   - `command.batch`: tudo ou nada, um passo de desfazer, refs `$ref`/`$step`. Histórico por snapshots
     (`imbl`).
   - Variables com coleções, modos, aliases e coleções estendidas (bom para N marcas).
     `variable.import` lê só o formato de variables do Figma; não lê DTCG nem Tokens Studio.
   - Codegen: CSS, HTML, React, React+Tailwind, Vue, SwiftUI, Compose, Flutter. Code Connect mapeia
     componente → import/props da biblioteca de código (`figcraft-cli connect publish`).
   - Build web (WASM) com host bridge (`?host=parent`, `window.figcraft`).
   - Plugins em WASM **sem rede**: não podem hospedar chat nem chamar conectores.
   - **Lacunas para o Eureka:**
     - Batch sem dry-run, preview ou rejeição.
     - Sem eventos de edição do documento (o tópico `document` de `events.poll` está reservado mas não emite).
     - Sem origem por camada (IA ou humano).
     - Sem lint de DS e sem bloqueio de componentes oficiais.
     - Sem cliente LLM nem conectores de saída no app.
2. **Benchmark de UX com IA** (Sonnet): Claude Design, Figma Make, v0, Lovable, Bolt, Subframe, Stitch,
   Cursor, Penpot, Builder, Onlook, Framer.
   - Nenhuma ferramenta resolve três coisas: origem por camada, proteção de edições manuais quando a
     IA refaz, e Gatekeeper do DS antes da revisão humana.
   - Esses três pontos são o diferencial do Eureka.
3. **Fontes do DS mapeadas:**
   - Figma DS N Marcas `FsU6PD5PxodVk0IVMWQrLP`: coleção Tokens WEB com modo por marca, coleção
     Layout Desktop/Mobile.
   - Team Components por marca, por exemplo TC Animale Web `lzVIxSC2ExCdvvYysBzRJ6`.
   - Biblioteca de código `@soma-monorepo/core-components`, sem acesso.
   - Artifact "DS Animale Web" (https://claude.ai/artifact/EgRkbc9we3UG4YhpbSK2Dx): tem
     `project/tokens.json` com 54 cores, 20 espaçamentos, 25 raios e tipografia do modo Animale,
     sincronizado em 27/10.
   - Regra de nome: variável Figma em minúsculas, com `/` e espaço trocados por `-`.
4. **Skill `eureka-ux-ai` v0** (commitada).
5. **Três protótipos de interação** (na página de revisão):
   - A: pedido na seleção com preview fantasma.
   - B: copiloto com cartões de proposta e diff por item.
   - C: quadro de exploração com 3 a 5 alternativas e o pipeline da Naty.
   - Recomendação: A para micro-ajustes, C para exploração, o cartão de B como superfície única de revisão.
6. **Teste do ciclo híbrido no app real** (`docs/eureka/spike_hybrid_loop.py`):
   - Importou os tokens do Animale como variables.
   - Claude gerou um card vinculado a tokens. O designer editou textos e pôs uma cor fora do DS.
   - Claude leu o estado (texto novo e o botão `#7A4B2A` sem vínculo), devolveu as cores aos tokens,
     adicionou o selo e preservou os textos.
   - Um ⌘Z desfaz o refino inteiro.
   - Lacunas confirmadas: proposta antes de aplicar e aviso do que mudou.

## Arquitetura recomendada (aguardando aprovação)

- **MVP: Claude orquestra fora do app** (Claude Code ou Desktop com MCPs do Eureka, Notion, Figma e
  DS Experts). O Eureka é a superfície visual.
- Novos no engine:
  1. Propostas: `proposal.create/diff/accept/reject` sobre uma cópia do documento.
  2. Eventos `document` e um delta "o que mudou desde o último turno".
  3. Origem por camada.
  4. Lint de DS.
  5. Adaptador de tokens do DS (sem cópia manual).
- Na UI: painel de propostas, preview no canvas e selo do Gatekeeper.
- Chat nativo no app fica para depois. Exige chave de API e cliente HTTPS, que é dependência nova
  com checagem de licença.
- Manter o fork compatível com o upstream: isolar o Eureka em crates novos.

## Decisões pendentes da dona do produto

1. Direção de interação: A, B, C ou uma combinação.
2. Onde fica o chat no MVP: fora do app via MCP (recomendado) ou painel nativo.
3. Governança do fork: manter as regras do upstream (Rust puro, sem webview, CI do
   `storytold/craft-common-actions`, que pode não rodar no fork) ou divergir.
4. Sinal verde para a Sprint 1.

## Plano proposto da Sprint 1 (rascunho, alinhar ao Notion)

| Id | Tarefa | Subagente · modelo | Aceite |
|---|---|---|---|
| S1-1 | Spike do loop com edição pelo mouse e roteiro demonstrável | Editor Engineer · Sonnet | Roteiro roda de ponta a ponta; edições manuais preservadas |
| S1-2 | Contrato e comandos de propostas (dry-run, diff, aceitar, rejeitar) | Architect · Opus revisa, Sonnet codifica | Aceitar = 1 desfazer; rejeitar sem rastro; testes + varredura de parâmetros inválidos |
| S1-3 | Eventos `document` + delta "o que mudou" | Editor Engineer · Sonnet | Claude recebe as camadas alteradas desde o último turno |
| S1-4 | Importador de tokens do DS (tokens.json → variables por marca) | Code Bridge · Sonnet; DS Governance · Opus revisa | Valores idênticos à fonte; DS não alterado |
| S1-5 | Protótipo da direção de UX escolhida | UX Interaction · Opus | Checklist da skill `eureka-ux-ai` |
| S1-6 | Contrato de conectores (Gatekeeper, agentes da Naty) | Connector Engineer · Sonnet | Revisado pela dona antes de integrar |

S1-2, S1-3 e S1-4 tocam crates diferentes e podem rodar em paralelo em worktrees isolados.

## Como reproduzir o teste

```sh
cargo build -p figcraft
PORT=<porta livre>; mkdir -p /tmp/eureka && echo $PORT > /tmp/eureka/port
cargo run -p figcraft -- --control $PORT &
python3 docs/eureka/spike_hybrid_loop.py /tmp/eureka generate <caminho>/tokens.json
python3 docs/eureka/spike_hybrid_loop.py /tmp/eureka manual   <caminho>/tokens.json
python3 docs/eureka/spike_hybrid_loop.py /tmp/eureka read     <caminho>/tokens.json
python3 docs/eureka/spike_hybrid_loop.py /tmp/eureka refine   <caminho>/tokens.json
```

O `tokens.json` vem do artifact "DS Animale Web" (arquivo `project/tokens.json`). Ele não está
commitado porque é material do DS, e o repositório não deve manter uma cópia.
