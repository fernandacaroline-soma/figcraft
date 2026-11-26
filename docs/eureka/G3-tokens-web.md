# G3: `export.tokensCss` para Web (Tarefa 4 da FILA)

Branch: `eureka/g3-tokens-web`, a partir de `eureka/g3-export-code` (Tarefa 1).

## O problema

No DS Web os tokens ficam em mais de uma coleção:

- **Tokens WEB**: 12 marcas.
- **Layout**: Desktop e Mobile.
- **Checkout e Account**: a coleção "Variable collection", com um só modo.

O `export.tokensCss` escrevia todo modo de toda coleção como `[data-brand="…"]`. Com isso o Layout virava `data-brand="mobile"`, o que é errado. A coleção de um modo virava uma "marca" falsa.

## O que mudou

- **Uma regra para a coleção de marca do arquivo** (`brand_collection`), revista depois da revisão do PC1:
  1. a coleção chamada **Tokens APP** ou **Tokens WEB** (`BRAND_COLLECTIONS`), do arquivo ou de biblioteca, a do arquivo primeiro;
  2. na falta delas, a com mais modos, a do arquivo primeiro no empate.

  Assim, um "Layout" local nunca toma a marca de uma Tokens APP de biblioteca.
- **A mesma coleção nos dois arquivos.** O parâmetro `collection` vale para o `export.tsx`, o `export.tokensCss` e o `export.code`. Com ele, o `data-brand` do TSX e o do CSS são sempre da mesma coleção.
- **As outras coleções com modos ganham atributo próprio**, com o nome da coleção pela mesma regra dos tokens:
  ```css
  :root,
  [data-layout="desktop"] { --grid-gutter: 24px; … }
  [data-layout="mobile"]  { --grid-gutter: 8px; … }
  ```
- **Coleção de um modo** sai só em `:root`, sem atributo, porque o valor é o mesmo em toda parte.
- **No TSX**, o modo de cada coleção definido na tela ou acima dela vira atributo da raiz: `<div data-brand="nv" data-layout="mobile">`. Antes, só a marca entrava e o resto virava nota.
- **Saída nova do `tokensCss`:** `attributes: [{collection, attribute, modes: [{mode, value}]}]`. `attribute` é `null` quando a coleção tem um modo. `brands` agora traz só a coleção de marca.
- **Novos avisos:**
  - duas coleções com o mesmo atributo, por exemplo "Layout" e "LAYOUT";
  - dois modos da mesma coleção com o mesmo valor, aviso que antes valia só para marcas.

## Nomes iguais entre App e Web

- A regra do nome da variável CSS é uma só (`ds_var_name`, pelo nome do token), então um token com o mesmo nome no App e na Web é a mesma variável.
- Comparei os snapshots do cache, só os nomes:
  - App: `tokens-app-9-marcas-textstyles-*`, 37 variáveis.
  - Web: `tokens-web-animale-farm-*`, 23 variáveis.
- Resultado:
  - 22 nomes estão nos dois e dão a mesma variável CSS;
  - nenhum nome diferente gera a mesma variável;
  - 1 só existe no Web: `Colors/Surface/Brand/Terciary`.
- Se um arquivo tiver as duas coleções, a colisão vira aviso. O teste `app_and_web_name_a_token_the_same` cobre isso.

## Validação

- `export_tsx`: 27 testes. Os 4 da primeira entrega usam um arquivo Web sintético: 9 marcas, Layout com Desktop/Mobile e uma coleção de um modo. Os 3 do ajuste do PC1 são:
  - `the_named_brand_collection_wins_even_from_a_library`: Tokens APP de biblioteca mais Layout e tema locais;
  - `a_chosen_brand_collection_is_the_same_in_the_tsx_and_the_stylesheet`;
  - `a_local_collection_with_more_modes_does_not_take_the_brand_from_tokens_app`.
  - `layout_modes_are_their_own_attribute_not_a_brand`
  - `the_screen_says_its_brand_and_its_layout`
  - `two_collections_on_one_attribute_are_a_warning`
  - `app_and_web_name_a_token_the_same`
- O teste da Tarefa 1 sobre a coleção de marca agora espera que uma coleção de um modo não gere atributo nem nota.
- Também rodei `codegen`, `inspect`, `variables_io`, `eureka` e o junk sweep.
- `clippy -D warnings` no engine; CI no branch.

## Limites

- **Snapshot Web incompleto.** O do cache só tem a Tokens WEB, com 23 variáveis e 2 marcas. O Layout (14 variáveis) e a coleção do Checkout não foram lidos do Figma; os testes usam dados sintéticos. Para exportar o Web real, gere um snapshot com as três coleções: leitura do Figma, só leitura.
- **Número sem uso conhecido sai em px.** É a regra atual do `token_roles`: só opacidade e peso ficam sem unidade. Uma contagem de colunas sairia `12px`. Isso pede um papel por escopo da variável (`scopes`), ainda não feito.
- **Alias entre coleções.** Uma variável do Layout que aponta para um token de marca é resolvida onde foi declarada. Em `:root`, ela fica com a marca padrão. No caso normal, com `data-layout` no mesmo elemento que `data-brand` ou dentro dele, a resolução está certa.
- **Coleção de marca com outro nome.** Um arquivo cuja coleção de marcas não se chama Tokens APP nem Tokens WEB cai na regra de mais modos. Nesse caso, passe `collection` ao `export.code`, que leva a escolha aos dois arquivos.

## Ajustes da revisão (PC1, 26/11)

- **Regra da marca pelo nome**, descrita acima. Com ela, o `export.code` usa a mesma coleção no TSX e no CSS.
- **Nomes repetidos.** Camadas "Card", "Card" e "Card2" agora geram Card, Card2 e Card22. Antes saíam duas funções `Card2`. Teste: `numbered_names_never_meet_a_layer_already_named_so`.
- **`export.codeMap`, duas mudanças:**
  - uma segunda entrada para o mesmo componente e linguagem vai para `invalid`. O `codeConnect.import` ficaria com a última.
  - o trabalho tem limite: 20 000 verificações (entradas × componentes de mesmo nome). Acima disso, `truncated: true`.

  Teste: `a_second_entry_for_one_component_is_pointed_out`.
