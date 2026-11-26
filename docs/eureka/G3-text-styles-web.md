# G3: text styles no `export.tokensCss` da Web (Tarefa 8 da FILA)

Branch: `eureka/g3-text-styles-web`, a partir de `claude/eureka-design-lead-epvzr2`.

## O que estava errado

Reproduzido com um arquivo Web sintético (Tokens WEB com marcas, Layout com Desktop/Mobile, text styles vinculados a variáveis):

- **Os text styles já saíam** na folha da Web, com os mesmos nomes do App (`--text-<estilo>-<campo>`). Nomes iguais para estilos iguais: a regra de nome (`slug` do estilo, `ds_var_name` da variável) é uma só para os dois arquivos. Não havia dois padrões de nome.
- **O que quebrava era a coleção de cada variável vinculada.** O código do text style só conhecia a coleção de marca:
  1. A regra dos estilos era sempre `:root, [data-brand]`. Um campo vinculado a variável do **Layout** (ex.: altura de linha por Desktop/Mobile) lia um `var()` que só se resolve no elemento onde a propriedade é declarada. Dentro de um `data-brand`, com o `data-layout` num elemento filho, o estilo ficava com o valor do layout padrão.
  2. O peso vindo do nome do estilo da fonte (`--<variável>-weight`) era escrito sempre como `[data-brand="<modo>"]`, com os modos da coleção da variável. Numa variável do Layout isso gerava `data-brand="mobile"`, a marca falsa que a Tarefa 4 já tinha removido das variáveis.
- A coleção de marca escolhida por `collection` também não chegava ao text style (só valia o padrão).

## O que mudou (`crates/engine/src/dev/export_tsx.rs`)

- `text_style_tokens(doc, brand)` recebe a coleção de marca do export (a mesma do TSX e do resto da folha).
- A regra dos text styles ganha, além de `:root, [data-brand]`, o atributo de cada coleção com modos de onde algum estilo lê um token: `:root, [data-brand], [data-layout] { ... }`. Sem coleção extra, a saída é igual à de antes.
- Os blocos `--<variável>-weight` usam o atributo da coleção da variável: marca em `data-brand`, outra coleção com modos no atributo próprio (`data-layout`), coleção de um modo só em `:root`.
- Nenhum nome mudou.

## Validação (rodada)

Testes novos em `crates/engine/src/export_tsx_tests.rs`, dados sintéticos (Tokens WEB, Layout):

- `a_web_file_gets_its_text_styles_in_the_stylesheet`: os 5 campos de cada estilo, família vinculada, peso por marca em `data-brand`, regra também em `[data-layout]`. **Falha sem a correção.**
- `a_weight_name_in_the_layout_collection_is_a_number_per_layout_not_per_brand`: peso por nome numa variável do Layout sai por `data-layout`, nunca `data-brand="mobile"`. **Falha sem a correção.**
- `app_and_web_name_a_text_style_the_same`: o mesmo conjunto de estilos, com a coleção chamada Tokens APP e Tokens WEB, dá os mesmos nomes `--text-*`. Passa também sem a correção: guarda a regra de nome única, não o bug.

Falhas confirmadas revertendo só `export_tsx.rs`. Filtros rodados: `export_tsx`, `variables`, `junk`; `cargo fmt` e `clippy -D warnings` no engine.

## Limites

- Variável vinculada que é **alias** de outra coleção com modos: só a coleção da variável vinculada entra na regra, não a do alvo do alias.
- Text styles de biblioteca continuam de fora (como em G3-text-styles-tokens).
- Variável vinculada de coleção que não está em `collections` sai como `var()` sem definição na folha.
- Não validado com o snapshot real do DS Web (só dados sintéticos).
