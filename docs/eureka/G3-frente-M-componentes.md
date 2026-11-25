# Frente M — telas com componentes da biblioteca DS Apps

Branch `eureka/m-componentes-biblioteca` (25/11). Faz parte do E17 (G3). A letra K já é a UI do app
no plano (E05 §6); esta frente é a M.

## O que muda

Até aqui as telas do quadro de exploração eram desenhadas por script, só com tokens. O Gatekeeper
apontava, com razão, "botão desenhado à mão em vez da instância oficial". Agora a rodada NV (Rodada
01, 13/11) é montada com instâncias dos componentes oficiais do DS Apps:

| Na tela | Componente DS Apps |
|---|---|
| Botão "Adicionar à sacola" / "Selecione o tamanho" | Button (Primário, Grande, sem ícone) |
| Seletor de tamanho PP–GG | Selector Size (Selector Button), Grande |
| Escala justo / fiel / folgado (B) | Choice Tag (Chips), Médio |
| Folha de tamanhos (E) | Bottom Sheet, com os tamanhos no slot e M selecionado |

O que não é componente (tela, coluna de conteúdo, galeria e textos) continua desenhado e ligado a
tokens Tokens APP. O banner do Concierge (C) fica com o nome "sem componente verificado": ninguém
confirmou ainda se o DS Apps tem um equivalente.

## Como os componentes chegam ao Eureka

1. `ds_components_snapshot.js` roda no Figma pelo MCP (`use_figma`), **só leitura**. Ele lê os
   conjuntos escolhidos e tudo de que as instâncias deles dependem (ícones, divisor, slots,
   Action Button, Snackbar…), só nas variantes usadas, e gera o JSON no formato da API REST
   (`/v1/files/:key/nodes`), com as variables vinculadas (todas as 9 marcas do Tokens APP).
2. O resultado sai em partes de 38 KB (limite de 48 KB por chamada) e `ds_components_snapshot.py
   assemble` junta tudo num arquivo em `eureka-cache/`, **fora do repositório**. Desta vez:
   `components-app-nv-round-bd88e36339f2.json`, com 16 componentes/conjuntos, 42 variantes e
   61 variables.
3. `file.import` abre esse JSON no FigCraft pelo importador REST que já existia. O script marca cada
   componente importado com `eureka.library` (DS Apps, arquivo, snapshot).

Nada é escrito no Figma ou no Notion. Nenhum componente, token ou regra do DS muda.

## Mudanças no motor (com testes)

- **Lint (`ds.lint`):** um valor fixo dentro de uma instância que o componente principal também tem é
  do componente (cabe ao time de DS), não da tela. Ele vai para a contagem `inherited` e não aparece
  como achado. Override feito na instância continua sendo achado. Na rodada: 0 achados da proposta,
  60 herdados.
- **Contexto do revisor:** `library` agora vem preenchido quando a importação registrou a origem
  (antes era sempre nulo). Camadas ocultas ficam de fora, para os estados internos dos componentes
  não se passarem por camadas da tela. Cada camada traz o pai, e as camadas de texto trazem o texto.

Testes: `the_lint_reports_an_instance_override_but_not_the_components_own_values`,
`the_reviewer_sees_which_library_an_instance_comes_from`,
`the_review_context_leaves_hidden_layers_out_and_carries_the_texts`.

## Antes e depois no Gatekeeper (proposta E, escolha simulada)

A escolha da E foi **só uma simulação** num arquivo local, para comparar os pareceres. A escolha real
da rodada continua com a designer.

- **Antes** (`demo-g3m/M4`, telas por script): com ressalvas. Button, Bottom Sheet e Chips eram
  FRAMEs (reconstrução manual de componente oficial), o contorno usava token com escopo vazio e a
  biblioteca de origem não vinha no pedido.
- **Depois, primeiro contexto** (`M5`): confirmou instâncias do DS Apps e lint vazio, mas leu como
  visíveis os estados ocultos do Bottom Sheet (loading, Snackbar, botão secundário). Isso levou à
  correção do contexto acima.
- **Depois** (`M6`): com ressalvas, agora sobre o design. Pede que o botão "fixo" use o componente
  fixo oficial (Action Button) na PDP, troque a galeria e as informações do produto por componentes,
  e mostre os estados de indisponível, erro e retorno ao adicionar. Também nota um `.HomeIndicator`
  duplicado, que vem da estrutura do próprio Bottom Sheet + Action Button do DS Apps.

## Limites conhecidos

- Os ícones do DS são fontes (`nv-icons`, `farm-icons`…) que não estão instaladas neste Mac. O
  FigCraft desenha o nome do ícone numa fonte reserva, e Montserrat também cai na fonte reserva.
- O FigCraft ainda não aceita variable em `width`/`height` (19 tamanhos de ícone), em `cornerRadius`
  geral, nem em mais de um fill. A importação avisa e mantém o valor.
- **Achado no DS Apps:** o conjunto `.Nested / Progress` tem o raio da moldura ligado a uma variable
  de outra biblioteca ("Tokens", modos XP…), e por isso o seletor de modos mostra uma coleção
  estranha. É do arquivo do DS, não foi alterado e fica para o time de DS avaliar.
- O snapshot é uma foto: se o DS Apps mudar, é preciso gerar de novo (o nome do arquivo leva o hash).
- Ainda falta a skill gerador-propostas de verdade, que hoje é substituída pelo script com as cinco
  propostas registradas no Notion.

## Rodar

```
python3 docs/eureka/spike_nv_round_ds.py WORKDIR eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-nv-round-bd88e36339f2.json
python3 docs/eureka/gatekeeper_bridge.py export-choice WORKDIR SCREEN_ID
```
