# G3: exportar a proposta escolhida como código (Tarefa 5 da FILA)

Branch: `eureka/g3-handoff-proposta`, a partir de `eureka/g3-tsx-ds-components`, que traz as Tarefas 1 a 4. Não mexi em `proposal.rs` nem nos arquivos `eureka_*` do PC1: o comando está num arquivo novo, `crates/engine/src/dev/export_handoff.rs`.

## O comando

`explore.exportCode {id, dir?, collection?}` é só leitura. `id` é a tela da proposta escolhida.

**Ordem.** O comando segue a mesma ordem do quadro e recusa pular etapas:

1. a designer escolheu a proposta (`explore.choose`);
2. a crítica foi revisada (`explore.critiqueReviewed`);
3. existe parecer (`explore.review`), o que já exige o passo 2.

**Saída.** Gera o mesmo que o `export.code` (Tarefa 1), `<Tela>.tsx` e `tokens.css`, com o TSX encabeçado por um resumo para o dev:

```
/*
 * Eureka: the proposal the designer chose, reviewed, for the developers.
 * Proposal A · Um (Disruptiva), board "Rodada NV"
 *   Tamanho antes do preço.
 * Verdict (DS Gatekeeper): com-ressalvas · Pode seguir com ajustes.
 *   - Padding sem token. [Regras DS Apps, Tokens]
 * Design critique: 2 finding(s), 1 open:
 *   - [baixa] falhas: Sem foto do avesso.
 */
```

Logo abaixo vem o cabeçalho de sempre do `export.tsx`, com marca, notas e valores sem token, e depois o código.

**Regras do resumo:**
- Achados com status "resolvido" ou "decisão da designer" ficam fora da lista de abertos.
- A ordem é da gravidade maior para a menor, e a lista vai até 20 itens; os demais são só contados.
- Um `*/` no texto do revisor é neutralizado, para não fechar o comentário.

O resultado traz ainda `proposal`, `review` e `critique: {findings, open}`.

## Para ligar ao cartão (PC1)

No cartão, depois do parecer, a ação "Exportar código" chama o comando assim:

```
explore.exportCode {id: <tela escolhida>, dir: <pasta escolhida>}
```

Depois, o cartão mostra `tokens` (o que veio como token) e `warnings` (o que ficou solto), como faz o botão do painel em `panels/code_export.rs`. A função `code_export::show` pode ser reaproveitada se ficar `pub(crate)`.

## Validação

- `the_chosen_proposal_goes_out_as_code_with_its_verdict_and_open_findings`:
  - recusa antes da escolha, antes do parecer e numa proposta não escolhida;
  - confere o resumo (proposta, parecer com item e fonte, achado resolvido fora);
  - o TSX traz a marca NV e os tokens;
  - duas chamadas dão o mesmo resultado (só leitura);
  - parâmetros inválidos são recusados.
- `comment_closers_in_the_reviewers_text_do_not_end_the_header`: um `*/` no parecer e na crítica não fecha o comentário.
- Também rodei `eureka`, `export_tsx`, o junk sweep e o `clippy -D warnings` no engine. O resultado do CI está no commit.

## Limites

- O resumo vai em inglês, como o resto do cabeçalho. O texto do revisor e da crítica sai como foi escrito, em pt-BR.
- Os itens do parecer são listados sem os `ids` das camadas.
- A ação no cartão ainda não existe. Fica com o PC1, conforme a FILA.
