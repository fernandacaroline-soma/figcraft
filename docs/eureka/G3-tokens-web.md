# G3: `export.tokensCss` para Web (Tarefa 4 da FILA)

Branch: `eureka/g3-tokens-web`, a partir de `eureka/g3-export-code` (Tarefa 1).

## O problema

No DS Web os tokens ficam em mais de uma coleção:

- **Tokens WEB**: 12 marcas.
- **Layout**: Desktop e Mobile.
- **Checkout e Account**: a coleção "Variable collection", com um só modo.

O `export.tokensCss` escrevia todo modo de toda coleção como `[data-brand="…"]`. Com isso o Layout virava `data-brand="mobile"`, o que é errado. A coleção de um modo virava uma "marca" falsa.

## O que mudou

- **Uma regra para a coleção de marca do arquivo** (`brand_collection`). É a coleção do próprio arquivo com mais modos; no Web, a Tokens WEB, e no App, a Tokens APP. Sem coleção própria, vale a cópia de biblioteca com mais modos. `export.tsx` e `export.tokensCss` usam a mesma regra, então o atributo que o TSX põe sempre tem bloco no CSS.
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

- `export_tsx`: 23 testes. Os 4 novos usam um arquivo Web sintético: 9 marcas, Layout com Desktop/Mobile e uma coleção de um modo.
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
- **Coleção de marca pelo número de modos.** Se outra coleção do arquivo tiver mais modos que a de marcas, escolha a de marcas com o parâmetro `collection` do `export.tsx`. O `tokensCss` segue a regra do arquivo.
