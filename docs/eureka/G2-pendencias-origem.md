# G2 — Pendências da origem por camada (outro computador)

Branch: `eureka/g2-origem-ajustes`, a partir de `claude/eureka-design-lead-epvzr2` no fork `fernandacaroline-soma/figcraft`. Sem merge e sem PR: a dona aprova a junção. A sessão principal trabalha ao mesmo tempo na ponte com o DS Gatekeeper (`docs/eureka/`, scripts Python); não mexa nesses arquivos.

## Leitura obrigatória

- `AGENTS.md`.
- `docs/eureka/E05-arquitetura-adrs.md`: ADR-009, em especial a Frente E (origem opt-in).
- `crates/engine/src/cmd/eureka.rs`: `record_origin`, `origin_of`, `ORIGIN_KEY`, `undo_ai`.
- `crates/ui-egui/src/panels/layers.rs`: `by_ai`, `ai_undoable` e o selo da linha da camada.

## Tarefa 1: merge de branch com a origem ligada

- **Problema.** Com `origin.track` ligado, cada edição grava `pluginData["eureka.origin"]` na camada. Ao juntar um branch do próprio app (comandos `branch.*`), duas versões dessa chave na mesma camada viram **conflito**, mesmo quando o designer não mexeu em nada parecido.
- **Regra.** Na junção, `eureka.origin` nunca é conflito. Para cada aspecto (chave do objeto JSON), vence o carimbo com a `revision` mais alta; se não der para comparar, vence o do branch que está entrando.
- **Onde.** Ache no código de branches do engine onde `plugin_data` entra na comparação de conflitos. Trate só a chave `eureka.origin`; o resto do `pluginData` segue a regra atual.
  - **Mudança mínima:** esse código é do projeto original.
  - **Deixe escrito no comentário o motivo:** a origem é informativa (ADR-009) e não pode travar uma junção.
- **Teste:** dois branches editam aspectos diferentes da mesma camada com a origem ligada. A junção não acusa conflito, e a origem final tem os dois aspectos.

## Tarefa 2: selo IA mais discreto

- **Problema.** Numa tela gerada pelo Claude, todas as camadas levam o selo "IA" no painel de camadas, e ele vira ruído.
- **Proposta.** Mostrar o selo cheio só quando a IA **mudou algo que já existia** (há `was` no carimbo, ou seja, dá para desfazer só a IA). Camada criada pela IA e não editada depois ganha um selo discreto: contorno, sem preenchimento, ou só o texto "IA" em cor terciária.
- **Onde.** `crates/ui-egui/src/panels/layers.rs` (desenho da linha) e, se precisar, um helper ao lado de `by_ai`. Cores de `theme::Tokens`, textos via `tr()` com en e pt-BR.
- **Verificação:** screenshot do painel de camadas com a demo `spike_proposals.py WORKDIR CACHE --app --steps=import,generate` seguida de `spike_loop.py WORKDIR CACHE --app`. Camadas só geradas ficam discretas; Button e Label, que a proposta mudou, ficam com o selo cheio.

## Regras de trabalho

- **Sem sobrecarregar o computador:** `cargo` com `-j2`, testes filtrados (`cargo test -p figcraft-engine --lib -- eureka` ou `-- branch`). Nunca `cargo xtask ci` nem a suíte inteira localmente.
- **CI no GitHub:** o workflow `Eureka` roda no push de `eureka/**`.
- **Antes de cada push:** rode `cargo clippy -p <crate> --all-targets -- -D warnings` nos crates que mudou.
- **Commits:** mensagens terminam com `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Limites:** sem merge, sem PR ao upstream, sem escrever no Notion e sem mexer no Figma.
- **Windows:** `python -X utf8` e `CARGO_TARGET_DIR` curto.
- **Cache de tokens de App:** peça à dona o arquivo `tokens-app-animale-farm-9a70a2961b9d.json` e salve fora do repo.

## Pronto quando

- Os testes da Tarefa 1 passam.
- O screenshot da Tarefa 2 mostra a diferença entre os dois selos.
- O CI do branch está verde.
- Traga o link do branch e os commits.
