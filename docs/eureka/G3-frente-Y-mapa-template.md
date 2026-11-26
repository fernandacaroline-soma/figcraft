# Frente Y: mapa de código das peças do template da PDP

Branch `eureka/y-mapa-template` (26/11).

## Objetivo

Na Frente X, a G (montada sobre o template oficial da PDP) saía como código com **56 valores sem
token**, porque 28 componentes do DS saíam desenhados como camadas. Isso valia para as peças do
template (galeria, informações, tamanhos e área fixa) e para tudo o que vem dentro delas. Faltava o
mapa de componente do DS para componente de código (o Code Connect, da tarefa 3 da fila) para essas
peças.

## O que foi feito

### 1. O mapa (`docs/eureka/mapa_y.py`)

- **Quais componentes:** o `export.codeMap` lista todos os componentes do DS no arquivo. São 36: as
  quatro peças do template, as peças `.Nested` que elas usam e os componentes do DS Apps.
- **Como vira o template:** para cada componente, o `inspect.componentCode` dá o nome de cada
  propriedade em código, e o template do mapa lê todas. Exemplo:
  `<PdpMainInfo sale={sale} indisponivel={indisponivel} colors={colors} tag={tag} />`.
- **Nomes:** o nome do componente de código é o nome do DS, sem acento e sem o prefixo `.Nested /`
  (`PdpProductGallery`, `SizeSelectionPdp`, `ActionButton`), como pede a regra de nomenclatura
  igual à do DS.
- **Onde fica:** em `eureka-cache/ds-apps-pdp.connect.json`, fora do repositório.

> **Os nomes de código, o pacote (`@azzas/ds-apps`) e os nomes das props são proposta.** Quem
> confirma é a Tecnologia. O mapa é só configuração: corrigir um nome é editar o arquivo, sem
> mexer no código do Eureka.

### 2. Três correções no motor, cada uma com teste que falha sem ela

| Problema | Exemplo na G | Correção | Teste |
|---|---|---|---|
| **Acento cortava a letra, e o id do Figma entrava no nome.** O gerador só aceitava letras ASCII. | "Indisponível" virava `indisponVel`, "Ícone" virava `cone`, "✍️ Texto#7:26" virava `texto726`. | A letra acentuada vira a letra-base (`indisponivel`, `icone`); o emoji separa palavras; o sufixo de id (`#7:26`) sai do nome. Vale para o nome do componente e das props, em todas as linguagens. | `names_in_portuguese_keep_their_accented_letters_and_drop_emoji_and_ids` |
| **A chamada perdia o que a proposta trocou dentro do componente, sem aviso.** | `<ActionButton vertical="Off" fixed="On" />` não levava os dois botões da G ("Comprar agora · M" e "Adicionar à sacola"), justamente o centro da proposta. | Cada chamada ganha `inside`: os textos, as variantes, os booleanos e a visibilidade trocados dentro da instância, comparados com a camada do componente que cada um espelha. O cabeçalho do TSX ganha uma nota para o dev. O texto que a própria chamada já passa (propriedade de texto do componente) não entra. | `what_an_instance_changed_inside_a_call_is_listed_for_the_developers` |
| **O componente de um instance swap não era importado.** | O Snackbar passava `botao2={<Icon />}` sem `import { Icon }`: o arquivo não compilava. | O export usa as importações que o Code Connect já calcula para a chamada, incluindo os componentes que os instance swaps colocam. | `a_component_an_instance_swap_places_in_a_call_is_imported_too` |

## Resultado na G (base do template, NV)

Rodada refeita do zero, no app (`demo-g3y/Y0-mapa.txt`):

| | Antes do mapa | Depois |
|---|---|---|
| Valores sem token | 56 | **0** |
| Componentes do DS desenhados como camadas | 28 | **0** |
| Chamadas a componentes de código | 0 | 5: `PdpProductGallery`, `PdpMainInfo`, `SizeSelectionPdp`, `Snackbar` e `ActionButton` |
| `export.codeMap` | | 36 ligados, 0 sem entrada, 0 inválidos, 0 chaves desconhecidas, 0 props sem leitura |

A nota que vai para o dev:
`Action Button · comprar: the call to ActionButton passes only its own properties; changed inside it: Botão: "Adicionar à sacola"; Botão: "Comprar agora · M"`.

`demo-g3y/Y1-export-G-sem-css.tsx` é o TSX da G, sem o CSS e sem valores do DS.

## O que isso mostra para o DS (decisão sua, nada alterado)

- **O Action Button não expõe os textos dos botões como propriedade.** Por isso, uma proposta que muda
  os rótulos só chega ao código por nota. Se o componente de código receber os rótulos (por exemplo
  `primaryLabel`), o mapa passa a lê-los. Isso depende de propriedade nova no DS ou de decisão da
  Tecnologia.
- **As peças `.Nested` como componentes de código** seguem o ponto do Gatekeeper da Frente T: para
  handoff, o certo é a instância do template, não as peças soltas. O mapa cobre as duas formas.

## Validação

- Os três testes novos falham sem a correção e passam com ela.
- `cargo clippy -p figcraft-engine --all-targets -D warnings`: limpo.
- Testes do motor: 2158 passam. Três testes do mapa mental (`board_mindmap_tests`) falham neste Mac,
  e também falham sem esta mudança (no CI eles passam). Não têm relação com esta frente.
- Rodada completa no app: G antes e depois, acima.

Figma só lido. Nenhum componente, token ou regra do DS foi alterado, e nada foi gravado no Notion
além do card E17.
