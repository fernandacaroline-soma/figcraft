# G1 — frentes que podem rodar em paralelo

Base de todas: fork `fernandacaroline-soma/figcraft`, branch `claude/eureka-design-lead-epvzr2`. Cada frente trabalha **no seu próprio branch** (`eureka/g1-<frente>`), criado a partir dele, e só mexe nos arquivos listados. Ninguém faz merge: a dona aprova. Os testes pesados rodam no CI do fork (workflow `Eureka`, dispara em push para `eureka/**`). Localmente: `cargo` com `-j2` e testes filtrados, nada de suíte completa.

Leitura obrigatória antes de começar: `AGENTS.md`, `docs/eureka/E05-arquitetura-adrs.md` (ADR-004 e ADR-009), `.claude/skills/eureka-ux-ai/SKILL.md`, `crates/engine/src/cmd/eureka.rs`.

## Frente K — cartão de proposta dentro do app (UI)

- **Objetivo.** Hoje a prévia da proposta mostra só a faixa "Prévia da proposta do Claude · nada foi aplicado" com Descartar e Aplicar (`crates/ui-egui/src/panels/versions.rs:528+`). O cartão de conflitos existe só no JSON (`manualConflicts`) e no texto da demo (`docs/eureka/demo-g1/5b-conflicts.txt`). Levar o cartão para o app.
- **O que o cartão mostra.**
  - **Resumo:** o `summary` de cada camada criada ou alterada, que já vem do `proposal.preview`.
  - **Conflitos com edição manual:** para cada um, "você: …" e "Claude: …", com a escolha **Fica a minha / Fica a do Claude**. O padrão é *Fica a minha*.
  - **Bloqueio:** item `needs-choice` bloqueia o Aplicar até haver escolha.
  - **Avisos de DS:** `dsWarnings` como aviso, nunca como erro.
- **Como.**
  - Trocar uma escolha refaz `proposal.preview` com `override: [ids]`; o `baseHash` muda junto.
  - Aplicar chama `proposal.apply`. Se vier `stale`, mostrar "a camada X mudou desde a prévia" e oferecer gerar de novo.
  - Textos via `tr()` com entradas em `locales/en.toml` e `pt-BR.toml`.
  - Cores de `theme::Tokens`.
  - UI fina: só lê o estado do engine e despacha comandos.
- **Arquivos.** `crates/ui-egui/src/panels/versions.rs` (ou um painel novo `panels/proposal.rs`, registrado no mínimo de lugares) e os dois `locales/*.toml`. Não mexer no engine; se faltar algo nele, anotar e pedir.
- **Pronto quando:**
  - screenshot da prévia com 3 conflitos (rode `python3 docs/eureka/spike_proposals.py WORKDIR CACHE --steps=import,generate,manual,propose`);
  - a escolha por item funciona;
  - `cargo clippy -p figcraft-ui-egui -- -D warnings` e o teste de idiomas passam no CI.

## Frente B — evento "documento mudou" + delta automático

- **Objetivo.** Hoje o Claude só sabe o que mudou quando pergunta (`document.changesSince`). Emitir um evento `document.changed` pelo canal de controle depois de cada comando que mude o documento, com `{revision, agent, changedIds (até 500), source: "human"|"agent"}`, para o Claude reagir à edição manual.
- **Arquivos.** `crates/engine/src/session/exec.rs` (já tem `record_event`), `crates/ui-egui/src/control.rs` ou o servidor de controle (repasse do evento), `docs/control-protocol.md`, testes novos em `crates/engine/src/eureka_tests.rs` (só acrescentar no fim do arquivo).
- **Cuidado.**
  - Não emitir em dry-run: o `grouped` desfaz tudo e o evento não pode vazar.
  - Medir o custo em documento grande: `changed_ids` com limite.
- **Pronto quando:**
  - o teste prova que dry-run não emite e que a edição manual emite `source: human`;
  - o CI passa.

## Frente C — snapshot de tokens com aliases (D3)

- **Objetivo.** Regerar o cache de tokens do Figma (DS N Marcas, arquivo `FsU6PD5PxodVk0IVMWQrLP`, coleção Tokens WEB, modos Animale e Farm), **mantendo os aliases** (`VARIABLE_ALIAS`) e trazendo as variables-alvo das coleções de origem.
- **Arquivos.**
  - Gravar o cache **fora do repo**, em `~/Claude/eureka-cache/`.
  - No repo, só o manifesto: `python3 docs/eureka/tokens_snapshot.py manifest CACHE docs/eureka/tokens-manifest.json`. Conferir com `check`.
- **Regras.**
  - Figma só leitura.
  - Não criar nem alterar variable.
  - Não versionar valores.
- **Pronto quando:**
  - o manifesto tem `aliasesKept: true`;
  - `spike_proposals.py` roda com o cache novo e a troca Animale → Farm continua certa (`6b-brand-farm.png`).

## Ordem e conflitos

- **K e C:** podem começar já. Arquivos não se cruzam.
- **B:** pode começar já. Toca `exec.rs`, que K e C não tocam.
- **Fica comigo (sessão principal):** juntar as frentes no branch principal, depois da aprovação da dona.
