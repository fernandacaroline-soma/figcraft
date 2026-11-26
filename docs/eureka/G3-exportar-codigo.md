# G3: correções do export e o botão "Exportar código"

Branch: `eureka/g3-export-code`, a partir de `eureka/g3-text-styles-tokens`, que se apoia em `eureka/g3-tokens-css` e `eureka/g3-export-tsx`.

## Parte 1: correções da revisão

Cada correção tem um teste em `crates/engine/src/export_tsx_tests.rs`. Os 8 testes novos falham sem as correções: as mudanças do engine foram retiradas, os testes rodados e as mudanças devolvidas.

| # | Onde | O que estava errado | O que mudou | Teste |
|---|---|---|---|---|
| 1 | `component_codegen.rs`, `web_with` | As regras das outras variantes de um conjunto eram geradas, mas os valores soltos delas eram descartados. | Os valores soltos das outras variantes entram nos avisos, cada um uma vez. Um conjunto de variantes passado ao `export.tsx` também não gerava nada (`describe` só aceita componente); agora ele sai pela primeira variante, com as outras como regras `data-*`. | `the_other_variants_loose_values_are_warnings_too` |
| 2 | `export_tsx.rs`, `brand_attr` | O `data-brand` vinha da primeira coleção com modo definido (a ordem do id), que podia ser um "Theme" claro/escuro em vez da coleção de marcas. | A marca vem da coleção de marcas: entre as coleções com modo definido, as do arquivo (as que o `export.tokensCss` escreve) vêm antes das de biblioteca, e depois a que tem mais modos. O parâmetro `collection` escolhe a coleção à mão. A saída ganha `notes`: marca vinda de uma coleção de biblioteca (o `tokens.css` deste arquivo não tem bloco para ela), outras coleções com modo definido, coleção pedida sem modo na camada. As notas também vão para o cabeçalho do arquivo. | `the_brand_comes_from_the_brand_collection`, `a_brand_from_a_library_collection_is_a_note` |
| 3 | `codegen/css.rs` | Com os quatro cantos iguais (0 nesta marca, por exemplo) e uma variável ligada a um canto só, o ramo uniforme lia só `cornerRadius` e perdia o token. | O `border-radius` só sai como valor único quando os quatro cantos estão ligados do mesmo jeito. Senão, sai canto a canto, com o token no canto ligado. | `a_radius_token_on_one_corner_is_not_lost` |
| 4 | `export_tsx.rs`, `export` | Duas camadas com o mesmo nome geravam duas funções e dois `import` com o mesmo nome. Um id repetido era exportado duas vezes. | Um id repetido sai uma vez. Um nome repetido ganha número (`PDPMobile2`, `pdp-mobile-2.css`) e uma nota. | `layers_with_one_name_get_numbered_names` |
| 5 | `export.tokensCss` | A mesma coleção passada duas vezes em `collections` (por nome e por id, por exemplo) saía em dois blocos, com avisos falsos de nomes repetidos. | A lista é deduplicada. | `a_collection_named_twice_is_one_block` |
| 6 | `docs/eureka/demo-g3/export-pdp-mobile.tsx` | O cabeçalho dizia "Warnings: none", mas os textos têm família e peso soltos. | Arquivo gerado de novo pelo `export.code` (abaixo). Agora o cabeçalho lista os valores soltos. | — |

## Parte 2: botão "Exportar código"

O comando novo é `export.code {ids?, dir?, collection?, collections?, textStyles?}`, só leitura.

- Ele roda `export.tsx` nas camadas e `export.tokensCss` no arquivo, e grava dois arquivos: `<Componente>.tsx` (nome da primeira camada) e `tokens.css`. Sem `dir`, os dois voltam como texto.
- A saída diz o que o dev recebe:
  - `tokens`: as variáveis que o TSX lê, cada uma com `defined`, que diz se o `tokens.css` a declara (um token de biblioteca, por exemplo, não é declarado);
  - `warnings`: os valores sem token;
  - `notes` e `cssWarnings`.

No app, a seção **Exportar** do painel direito, com uma camada selecionada, ganha o botão **Exportar código** (`crates/ui-egui/src/panels/code_export.rs`).

- No desktop ele pede uma pasta e grava os dois arquivos. No navegador, baixa os dois.
- Depois, a seção mostra:
  - os arquivos gravados e a marca;
  - **Como tokens (N)**: a lista de tokens; um token que o `tokens.css` não declara aparece em amarelo;
  - **Sem token (N)**: camada, campo e valor de cada valor solto.
- Os rótulos estão nas chaves de en.toml e pt-BR.toml, todas acrescentadas no fim do bloco `[ui]`.

## Validação

- Testes filtrados: `export_tsx` (19, sendo 8 novos), `codegen`, `inspect`, `style_vars` e `every_command_survives_junk_params`.
- `cargo clippy -p figcraft-engine -p figcraft-ui-egui --all-targets -- -D warnings`.
- App real: a demo PDP Mobile foi reconstruída e exportada pelo botão, com screenshot.
- Os números de cada rodada estão no commit e na nota de entrega.

## Limites

- Os `tokens` são lidos do texto do TSX (`var(--…)` fora dos comentários), não das ligações da camada.
- Um token ausente do `tokens.css` só aparece marcado; o export não o acrescenta.
- O relatório no painel fica até a seleção mudar. Ele não fica guardado no arquivo.
- A marca padrão segue uma regra simples: coleção do arquivo e depois mais modos. Num arquivo com duas coleções do arquivo e muitos modos cada, use `collection`.
- `export.tsx` ainda não usa `--text-<estilo>-*`.
