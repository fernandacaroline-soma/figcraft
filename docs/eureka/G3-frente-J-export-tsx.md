# G3 — Frente J: export TSX com tokens do DS (outro computador)

Branch: `eureka/g3-export-tsx`, a partir de `claude/eureka-design-lead-epvzr2` no fork `fernandacaroline-soma/figcraft`. Sem merge e sem PR: a dona aprova a junção. A sessão principal trabalha ao mesmo tempo no quadro de exploração e na ligação com o gerador-propostas; **não mexa em `crates/engine/src/cmd/eureka*.rs` nem em `docs/eureka/spike_*.py`**.

## Objetivo

A tela que a designer aprovou sai como **TSX** que a Tecnologia (o 007) consegue usar, com os **tokens do DS pelo nome**, nunca com valores soltos. Decisão já tomada (ADR-003): o documento do Eureka é a fonte; o TSX é **saída gerada**, nunca editado à mão como fonte.

## Leitura obrigatória

- `AGENTS.md`.
- `docs/eureka/E05-arquitetura-adrs.md`, ADR-003.
- O codegen que já existe: `crates/engine/src/dev/codegen.rs`. Ele já gera React, Tailwind, Vue e outros. Entenda como sai o React hoje e como ele trata cor, espaçamento e texto.
- `docs/eureka/demo-app/` e `docs/eureka/demo-g3/`: as telas do DS Apps que servem de teste.

## O que fazer

1. **Tokens por nome.** Quando uma propriedade está vinculada a uma variable (`bound`), o TSX usa o nome do token, nunca o valor resolvido.
   - **Forma:** variável CSS derivada do nome. Ex.: `Colors/Surface/Brand/Background` → `var(--colors-surface-brand-background)`; `Spacing/sp-md` → `var(--spacing-sp-md)`.
   - **Regra de conversão:** uma função só, documentada e testada (barra vira hífen, minúsculas, espaços e caracteres especiais normalizados).
   - **Valor solto:** se a propriedade não tem vínculo, o valor sai como está e o export lista o caso num bloco de **avisos** no topo do arquivo. O 007 enxerga a dívida.
2. **Modo de marca.** A tela tem um modo explícito, ex. Tokens APP = "Animale (CERTO)". O componente raiz recebe um atributo `data-brand="animale"` (nome do modo normalizado), e as variáveis CSS vêm daí. Não gere os valores das variáveis: eles vêm do DS da Tecnologia.
3. **Comando.** Exponha como comando do engine: `export.tsx {ids? (default a seleção)} → {code, warnings}`.
   - **Sem diálogo:** só retorna o texto (regra do projeto: comandos programáticos não abrem diálogo).
   - **Se o codegen já tiver comando de React:** reaproveite e acrescente uma opção de tokens, em vez de duplicar.
4. **Fora do escopo agora:** importação TSX → Eureka (é o "spike de importação limitada" do ADR-003, com critério próprio), Code Connect de componentes e estilos de texto completos (fonte, peso).

## Testes

- Unidade: a conversão de nome de token para variável CSS (casos com espaço, acento, barras), valor solto vira aviso, modo vira `data-brand`.
- Uma tela da demo do DS Apps exportada sem nenhum valor solto, com o arquivo salvo em `docs/eureka/demo-g3/export-*.tsx` para a dona ver.

## Regras de trabalho

- **Sem sobrecarregar o computador:** `cargo` com `-j2`, testes filtrados. Nunca `cargo xtask ci` nem a suíte inteira localmente.
- **Antes de cada push:** rode `cargo clippy -p <crate> --all-targets -- -D warnings` nos crates que mudou.
- **CI no GitHub:** o workflow `Eureka` roda no push de `eureka/**`.
- **Nunca `unwrap`/`expect`/`panic`** fora de teste.
- **Sem JS ou TS no app.** Gerar TSX como texto de saída é permitido.
- **Commits:** mensagens terminam com `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Limites:** sem merge, sem PR ao upstream, sem Notion e sem Figma.
- **Windows:** `python -X utf8` e `CARGO_TARGET_DIR` curto.
- **Cache de tokens de App:** `tokens-app-animale-farm-9a70a2961b9d.json`, fora do repo (já está nessa máquina).

## Pronto quando

- O comando `export.tsx` existe, com testes, e o CI do branch está verde.
- Um `.tsx` de exemplo de uma tela do DS Apps está em `docs/eureka/demo-g3/`, sem valores soltos.
- Traga o link do branch, os commits e o arquivo de exemplo.
