# Frente P: snapshot maior do DS Apps e propostas disruptivas

Branch `eureka/p-snapshot-disruptivas` (25/11). Faz parte do E17 (G3) e vem depois da Frente O.

## O problema

Nas frentes N e O, o snapshot do DS Apps tinha só os 10 componentes da Rodada 01, com poucas
variantes. Isso trazia dois limites:

- **A crítica não via estados que faltam.** O Button vinha só como "desabilitado: não" e
  "carregando: não", então não havia nada a cobrar.
- **Não dava para desenhar disruptivas.** As propostas não tinham área de ação fixa, mensagens nem
  cabeçalhos de seção. A skill v0.3.0 pede 3 disruptivas por rodada, e nenhuma tinha sido testada.

## O que muda

### 1. Snapshot maior, lido do DS Apps sem alterar nada

`ds_components_snapshot.js` ganhou mais componentes de origem:

- **Button:** os 3 tipos, cada um com os estados de desabilitado e de carregamento.
- **Já existiam:** Choice Tag, Selector Size e Bottom Sheet.
- **Novos:** Action Button, Snackbar, Tipbar e Section Heading.

Ficaram de fora Price Bar e Complete o look. Eles puxam a Bottom Bar e o Product Card inteiros,
cerca de 570 KB a mais.

**Resultado:** 563 KB de JSON, com 8 conjuntos principais e mais as peças internas de que eles
dependem, e 67 variables. O arquivo fica em
`eureka-cache/components-app-frente-p-295d7a197126.json`, fora do repositório, como os snapshots
anteriores. O Figma foi só lido.

**Como o snapshot chega ao Mac sem cópia manual.** Cada leitura do Figma devolve no máximo 48 KB,
e o JSON passa disso. Por isso:

1. o script comprime o JSON (LZW em base64, cerca de um terço do tamanho);
2. o JSON comprimido sai em 4 partes, em vez de 14;
3. `ds_components_snapshot.py assemble` junta as partes e descomprime.

A compressão foi testada ida e volta: o arquivo reconstruído é idêntico ao original.

### 2. Medição da crítica mais precisa

Com o snapshot maior, apareceram dois erros na medição, já corrigidos e cobertos por teste.

- **Nome tomado por estado.** Um ícone chamado `error-circle` era tratado como "estado de erro".
  - Agora uma propriedade liga/desliga de estado (Desabilitado, Loading) cobra o valor ligado.
  - Nas outras propriedades, um valor só conta como estado se ele inteiro for um estado (Erro,
    Sucesso, Vazio, Loading…).
  - Assim, no Snackbar, Status cobra só Erro, e não Brand, Neutro, Informação ou Alerta.
- **Área de toque repetida.** A medição criava um achado para cada tamanho. Agora cria um por
  componente, com os tamanhos encontrados na tela.

### 3. Três propostas disruptivas (teste)

O arquivo `rodada_nv01_disruptivas.json` acrescenta F, G e H ao quadro da Rodada 01. O título do
quadro passa a dizer **"+ 3 de teste do Eureka"**.

Essas três propostas foram escritas para testar o formato e o snapshot. Elas não são saída da
skill. As propostas reais vêm da skill, e quem escolhe é a designer.

| | Proposta | Componentes do DS |
|---|---|---|
| F | Tamanho resolvido antes da galeria | Bottom Sheet modal, Choice Tag, Snackbar de informação |
| G | Comprar agora no tamanho de sempre | Action Button horizontal fixo, Snackbar de sucesso |
| H | Provar na loja antes de comprar | Section Heading, Action Button vertical fixo |

O script `rodada_n.py` também mudou:
- aceita mais de um arquivo de propostas;
- aceita `no_button`, para propostas que trazem a própria área de ação.

**Resultado:** 8 propostas desenhadas (`P1`), com **lint 0 nas propostas** e 93 valores herdados
dos componentes do DS.

## Medição: snapshot antigo × novo (`demo-g3p/`)

Achados medidos pelo Eureka em cada tela:

| Proposta | Snapshot antigo | Snapshot novo |
|---|---|---|
| A | 2 | 4 |
| B | 4 | 6 |
| C | 2 | 4 |
| D | 2 | 4 |
| E | 1 | 3 |
| F (disruptiva) | — | 8 |
| G (disruptiva) | — | 6 |
| H (disruptiva) | — | 5 |

**O que mudou com o snapshot novo:**
- Em todas as telas apareceram os estados desabilitado e carregando do botão principal.
- Na G, a crítica pede o Snackbar de erro.
- Na F, apareceu um achado real de contraste. O subtítulo do Bottom Sheet fica com **4,1:1 no modo
  NV**, e o WCAG AA pede 4,5:1. Esse problema está no componente do DS, não na proposta. Fica para
  a liderança do DS avaliar; nada foi alterado.

## Crítica completa da G (`P2`, `P-critica-G.txt`)

A crítica teve 11 achados:

- **Por severidade:** 1 crítico, 1 alto, 7 médios e 2 baixos.
- **Por origem:**
  - 6 medidos pelo Eureka;
  - 5 do roteiro da skill.
- **Mais grave:** o tamanho de sempre sem estoque, de severidade crítica.
- **Do roteiro, também:**
  - cliente sem compra anterior;
  - dois botões parecidos lado a lado;
  - compra direta no checkout do app, como pergunta à engenharia;
  - métrica de conversão com a taxa de troca como controle.

Antes da revisão da crítica, o Gatekeeper foi recusado, como manda a skill. O status marcado e o
clique em "Revisei a crítica" foram só um teste do cartão.

## Rascunho para a skill

`rascunho-skill-references-eureka.md` é o rascunho da referência `references/eureka.md`, para você
levar à Naty. O repositório dela não foi alterado. O rascunho cobre:

1. o catálogo, no lugar do `search_design_system`;
2. a entrega da rodada em `screen`;
3. a escolha pelo cartão;
4. a crítica com o que o Eureka mede;
5. a trava antes do Gatekeeper.

## Limites

- **Componentes que ficaram de fora:** Price Bar, Complete o look, Product Card e Top Bar,
  porque são pesados demais para o snapshot.
- **Tipbar:** está no snapshot, mas sem propriedade de texto. Por isso não entrou nas propostas.
- **Área de toque:** continua sem saber se o elemento é tocável. O Section Heading sem ação
  (327 × 32 pt) aparece como achado de severidade baixa, com "se for tocável".
- **Contraste:** só mede o fundo do contêiner preenchido mais próximo.

## Testes

- `the_eureka_measures_missing_states_contrast_in_the_brand_and_small_touch_targets` ganhou três
  verificações:
  - um ícone chamado `error-circle`, que não pode contar como estado;
  - uma propriedade Status com Brand e Erro, que só deve cobrar Erro;
  - duas Tags iguais, que devem gerar um único achado de toque.
- Os 50 testes do Eureka passam.

## Rodar

```
python3 docs/eureka/rodada_n.py WORKDIR eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-frente-p-295d7a197126.json docs/eureka/rodada_nv01_propostas.json docs/eureka/rodada_nv01_disruptivas.json
python3 docs/eureka/medicao_p.py WORKDIR <ids das telas>
python3 docs/eureka/rodada_o.py WORKDIR <id da tela G> docs/eureka/rodada_nv01_critica_G.json
```
