# G2 — Frente F: lint de DS no canvas

Branch: `eureka/g2-lint`, criado a partir de `claude/eureka-design-lead-epvzr2` no fork `fernandacaroline-soma/figcraft`. Sem merge e sem PR: a dona aprova a junção. A sessão principal cuida da Frente E (origem por camada) ao mesmo tempo, em outros arquivos.

## Objetivo

O Claude (e depois o cartão) aponta onde uma tela foge das regras do DS.
- **Só avisa:** nunca bloqueia nem corrige sozinho.
- **Cita a fonte:** cada regra cita a fonte oficial dela (link do Notion ou do Figma), conforme a precedência Notion > Figma > decisões aprovadas.
- **Sem regra inventada:** nenhuma regra nova sem fonte. Regra sem fonte escrita não entra; vira pergunta para a dona.

## Leitura obrigatória

- `AGENTS.md`: sem `unwrap`/`expect`/`panic`, tudo é comando, camadas.
- `docs/eureka/E02-fontes-governanca-ds.md`: mapa de autoridade e onde cada regra mora.
- `docs/eureka/E05-arquitetura-adrs.md`: ADR-002 (tokens), ADR-009 (edição manual) e a seção 7 (decisões, em especial **D4: escopo de variable é aviso, não erro**).
- `crates/engine/src/cmd/eureka.rs`: `ds_warnings` já detecta valor solto em cor, tamanho de fonte, gap, padding e raio. O lint **estende isso**, não duplica.
- **Notion, só leitura:**
  - Fluxo Oficial de Criação e Alteração;
  - Playbook de Aprovação;
  - página DS N Marcas — Web;
  - Critérios de Escalonamento.

## Passo 1: tabela de regras (antes de codar)

Escreva `docs/eureka/G2-F-regras.md` com uma linha por regra candidata. Colunas:
- regra;
- fonte (link + trecho curto);
- o que dá para checar mecanicamente no documento;
- severidade (sempre "aviso" nesta fase);
- se entra agora.

Candidatas que já sabemos ter fonte ou decisão:
1. **Valor solto** onde existe token (cor, espaçamento, raio, tamanho de fonte). Já existe em `ds_warnings`.
2. **Variable fora do escopo** (ex.: Spacing com escopo GAP usado em padding). D4 decidiu que é aviso.
3. **Marca não escolhida na tela.** O frame raiz não tem modo explícito da coleção de marca e herda o padrão do Figma, que hoje é Farm. Achado da Frente C, registrado no E05 (D3).
4. **Token de marca usado em componente original fora de teste** ("componente original fica neutro; modo só em teste", Notion DS N Marcas — Web).

Qualquer outra regra só entra com fonte. Se a fonte não for clara, liste como pergunta no fim do arquivo.

## Passo 2: comando no engine

- **Novo comando** `ds.lint {ids? (default: página atual), limit?}` → `{findings: [{id, name, rule, message, source, prop?, value?}], counted}`.
  - Somente leitura (`undoable` false). Não muda o documento.
  - `message` em pt-BR, legível para designer, no estilo do `summary` das propostas.
- **Arquivo:** `crates/engine/src/cmd/eureka_lint.rs`, com o registro mínimo no `cmd.rs`. Testes no fim de `crates/engine/src/eureka_tests.rs` (só acrescentar).
- **Reaproveitar** `ds_warnings` e os helpers de `eureka.rs`, mudando a visibilidade para `pub(crate)` se preciso. Não mexa na lógica de propostas, nem em `session/exec.rs`: essa é a Frente E.
- **Limites:** número de achados, profundidade e tamanho da página. Entrada tratada como hostil (há o sweep de parâmetros em `tests.rs`).

## Passo 3: demo

- `docs/eureka/spike_lint.py`: abre o card da demo (`spike_proposals.py --steps=import,generate,manual`), roda `ds.lint` e escreve o relatório legível.
- **Esperado:** acusar o preço 18 solto, o botão #7A4B2A e o rótulo #FFFFFF. Também acusar o Spacing em padding, se a demo usar.

## Regras de trabalho

- **Sem sobrecarregar o computador:** `cargo` com `-j2`, testes filtrados (`cargo test -p figcraft-engine --lib -- eureka`). Nunca `cargo xtask ci` nem a suíte inteira localmente.
- **CI no GitHub:** o workflow `Eureka` roda ao fazer push de `eureka/**`.
- **Commits:** mensagens terminam com `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Limites:** sem merge, sem PR ao upstream, sem escrever no Notion e sem alterar nada no Figma.
- **Windows:** `python -X utf8` e `CARGO_TARGET_DIR` curto.
- **Cache de tokens:** use o novo, `tokens-web-animale-farm-e49cc88eb5dc.json`, fora do repo.

## Pronto quando

- A tabela de regras está escrita, com fonte em cada linha e as perguntas listadas.
- O comando `ds.lint` existe, com testes, e o CI do branch está verde.
- O relatório da demo mostra os achados esperados.
