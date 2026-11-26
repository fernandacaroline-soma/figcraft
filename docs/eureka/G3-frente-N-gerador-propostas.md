# Frente N — o gerador de propostas desenha no Eureka, e a designer escolhe pelo app

Branch `eureka/n-gerador-propostas` (25/11). Faz parte do E17 (G3). Segue a Frente M.

## O problema

Até a Frente M, as telas do quadro eram montadas por um script com ids de camadas e com cada
proposta escrita à mão. A skill gerador-propostas (`Soma-UXDesign/gerador-propostas`, v0.3.0)
desenha no Figma pela própria receita (`references/figma-ds.md`): busca no DS, instâncias de
componentes oficiais e tokens. Ela não conhecia o Eureka e não tinha como dizer "use o Button
Primário com este texto" sem saber o id de cada camada. E a escolha só acontecia por comando.

## O que muda

### 1. Tela descrita pelo que ela é (`explore.board` com `screen`)

Uma proposta agora pode trazer `screen` em vez de `commands`. A tela é descrita do jeito que a
designer e a skill falam do DS:

```json
{"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Surface/Background", "children": [
  {"image": "Galeria", "height": 360, "fill": "Colors/Surface/Container"},
  {"text": "Vestido midi de alça", "size": "Font/Size/fs-xs", "color": "Colors/Content/01"},
  {"component": "Button", "variant": {"Tipo": "Primário", "Tamanho": "Grande"},
   "props": {"Texto": "Adicionar à sacola", "Ícone": false}, "width": "fill"},
  {"component": "Bottom Sheet", "dock": "bottom", "props": {"Título": "Escolha o tamanho"},
   "layers": {"Botão Primário": {"Texto": "Adicionar à sacola"}},
   "slot": [{"frame": "Tamanhos", "layout": "horizontal", "children": ["…"]}]}
]}
```

- **Componentes** pelo nome do conjunto e pelos valores de variante. **Propriedades** pelo nome
  curto, sem o emoji e sem o `#id` que o Figma acrescenta (`"✍️ Texto#7:26"` vira `Texto`). Quando
  uma propriedade de texto e uma booleana têm o mesmo nome (`✍️ Subtítulo` e `👁️ Subtítulo`, no
  Bottom Sheet), o tipo do valor decide qual delas usar.
- **Instâncias aninhadas** (`layers`) e **conteúdo de slot** (`slot`). O placeholder do slot fica
  oculto, porque é uma camada herdada do componente.
- **Cores, espaçamentos e tamanhos de fonte** sempre por nome de variable. Um número solto é
  aceito, mas o lint aponta.
- `dock: "bottom"` fixa um bloco na base da tela (folha, barra de ação).
- **Tudo é conferido antes de desenhar.** Componente, variante, propriedade, camada ou token com
  nome errado recusa o quadro inteiro e devolve as opções que existem, por exemplo "`Button` tipo é
  um de: Primário, Secundário, Terciário". Nada fica desenhado pela metade.
- A proposta guarda `ds.components` e `ds.variables`: o que ela usou, para a "Aderência preliminar
  ao DS" da skill e para o revisor.

### 2. `explore.catalog`

Lista o que uma descrição pode usar, com os nomes que ela usa: componentes e conjuntos do arquivo
(opções de variante, propriedades com tipo e padrão, quantos slots) e as variables. No Eureka, ele
faz o papel do `search_design_system` da receita da skill. Os blocos internos (`.Nested`, `_Base`)
ficam de fora.

### 3. Botão "Escolher esta proposta"

O cartão que aparece ao selecionar uma proposta no quadro ganhou o botão **Escolher esta
proposta**, que roda `explore.choose`: a escolhida fica marcada como "Escolhida", as outras como
"Não escolhida", e nada é apagado. O cartão também mostra os componentes do DS que a proposta usa.
Depois da escolha, o botão some e aparece "Ainda sem parecer.", até o Gatekeeper responder.

### 4. Lint: camadas ocultas dentro de instâncias ficam de fora

Um estado do componente que não aparece na tela (placeholder de slot, loading, erro) é do
componente, não da tela. Na rodada, isso tirou da contagem os 5 placeholders ocultos dos slots.

## Rodada NV 01 gerada pela skill (`demo-g3n/`)

`rodada_nv01_propostas.json` é a saída da skill no formato acima: as cinco propostas da Rodada 01
(13/11), já registradas no Notion, agora descritas por componentes. A tela base da PDP é a mesma
para todas, e cada proposta acrescenta os seus módulos, como na receita da skill. `rodada_n.py`
expande essa base e entrega o quadro ao Eureka.

- **N2:** o quadro, com 5 propostas.
- **N3:** o cartão com o botão "Escolher esta proposta".
- **N4:** o cartão depois do clique no app.
- Resultado (`N-rodada.txt`):
  - A a E usam Button, Selector Size, Choice Tag (B), Button Secundário (D) e Bottom Sheet com os
    tamanhos no slot (E);
  - **lint com 0 achados da proposta** e 55 valores fixos herdados dos componentes do DS.
- O clique em A foi **um teste do botão** num arquivo local. A escolha real da rodada continua com
  a designer.

Nada foi gravado no Figma nem no Notion. Nenhum componente, token ou regra do DS mudou.

## O que fica do lado da skill (não alterado)

O repositório `gerador-propostas` é da Naty e não foi mexido. Para a skill desenhar no Eureka,
falta uma referência própria (`references/eureka.md`) dizendo três coisas:

1. com o Eureka aberto, ler `explore.catalog` no lugar do `search_design_system`;
2. entregar cada proposta como `screen` no formato acima, dentro de `explore.board`;
3. depois da escolha, que agora é feita pelo botão, seguir para a crítica e o Gatekeeper como já
   faz hoje.

Essa mudança precisa ser combinada com a dona da skill.

## Limites

- O snapshot do DS Apps só tem os 10 componentes da rodada. Para propostas mais livres (galeria,
  informações do produto, Action Button fixo), é preciso gerar um snapshot maior com
  `ds_components_snapshot.js`.
- A rodada usou só incrementais, como a Rodada 01 registrada. A skill v0.3.0 pede 3 disruptivas, e
  o formato já aceita `type: "disruptiva"`.
- Ainda não existe a etapa de crítica antes do Gatekeeper (Frente O).
- Ícones e Montserrat continuam na fonte reserva, como na Frente M.

## Testes

- `a_described_screen_is_drawn_with_the_librarys_components_and_tokens`: variante certa, textos no
  topo e em instância aninhada, slot preenchido, folha na base e lint sem valor solto.
- `a_misspelt_name_refuses_the_board_with_what_exists_and_draws_nothing`: 8 erros de nome, cada
  um com as opções que existem e o documento intacto.
- `the_catalog_lists_components_by_the_names_a_description_uses`.

## Rodar

```
python3 docs/eureka/rodada_n.py WORKDIR eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-nv-round-bd88e36339f2.json docs/eureka/rodada_nv01_propostas.json
```
