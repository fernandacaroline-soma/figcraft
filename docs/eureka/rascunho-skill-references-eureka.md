# Rascunho para a skill gerador-propostas: `references/eureka.md`

> **Rascunho para a Fernanda levar à dona da skill.** Ele não foi aplicado no repositório
> `Soma-UXDesign/gerador-propostas` e nada muda na skill sem o acordo dela. O texto abaixo é a
> proposta de referência, escrito no tom e no formato das outras referências da skill.

---

# Eureka — desenhar as propostas no Eureka Design

Use esta referência quando a exploração acontece no **Eureka Design** (o editor com o DS da SOMA)
em vez do Figma. A receita do Figma (`figma-ds.md`) continua valendo quando o Eureka não estiver
aberto.

## 1. O que o DS oferece (no lugar do `search_design_system`)

Rode `explore.catalog`. Ele lista os componentes e as variables do DS carregado, com os nomes que a
descrição da tela usa:

- **componentes:** opções de cada variante, propriedades com tipo (texto ou liga/desliga) e
  padrão, e quantos slots o componente tem;
- **variables:** cores, espaçamentos e tamanhos de fonte, pelo nome do token (por exemplo
  `Colors/Content/01` e `Spacing/sp-xs`).

Não invente componente, variante, propriedade ou token. Se o que a proposta pede não existe no
catálogo, escreva isso na proposta como **novo padrão potencial** e desenhe com o que existe.

## 2. Entregar a rodada (`explore.board`)

Cada proposta leva `name`, `type` (`incremental` ou `disruptiva`), `mechanism`, `summary`,
`confidence` e `screen`. A `screen` descreve a tela pelo que ela é:

```json
{"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Surface/Background", "children": [
  {"image": "Galeria", "height": 360, "fill": "Colors/Surface/Container"},
  {"text": "Vestido midi de alça", "size": "Font/Size/fs-xs", "color": "Colors/Content/01"},
  {"component": "Button", "variant": {"Tipo": "Primário", "Tamanho": "Grande"},
   "props": {"Texto": "Adicionar à sacola", "Ícone": false}, "width": "fill"},
  {"component": "Action Button", "variant": {"Vertical": "Off", "Fixed": "On"}, "dock": "bottom",
   "layers": {"Botão Primário": {"Texto": "Comprar agora"}, "Botão Secundário": {"Texto": "Adicionar à sacola"}}},
  {"component": "Bottom Sheet", "dock": "bottom", "props": {"Título": "Escolha o tamanho"},
   "slot": [{"frame": "Tamanhos", "layout": "horizontal", "children": ["…"]}]}
]}
```

- **Componente:** pelo nome do conjunto, mais os valores de variante.
- **Propriedades:** pelo nome curto, sem emoji e sem `#id`.
- **`layers`:** muda as propriedades de uma instância de dentro do componente, como os botões de
  uma área de ação.
- **`slot`:** recebe o conteúdo do slot.
- **`dock: "bottom"`:** fixa o bloco na base da tela.
- **Cor, espaço e fonte:** sempre pelo nome da variable.
- **Imagem:** é um retângulo de exemplo, sem foto de produção.

O Eureka confere tudo antes de desenhar. Um nome errado recusa a rodada inteira e devolve as opções
que existem. Corrija e entregue de novo.

Mantenha a regra da skill: de 3 a 10 propostas, **3 disruptivas**, e cada proposta parte da mesma
tela base da jornada.

## 3. Escolha

A designer escolhe no próprio quadro, pelo botão **Escolher esta proposta** do cartão. Não escolha
por ela. A escolha marca a proposta e mantém as outras como registro da rodada.

## 4. Crítica (Fase C)

Depois da escolha, rode `explore.critique` na proposta escolhida, com os achados do
[roteiro de crítica](critica.md). Cada achado tem:

- `area`
- `scenario`
- `consequence`
- `severity`
- `evidence` (fonte, ou `inferência`)
- `recommendation`

Inclua também `refined` (a proposta refinada) e `open` (as decisões em aberto).

O Eureka acrescenta sozinho, com a evidência "Eureka (medido)":
- estados do DS que a tela não mostra;
- contraste do texto na marca da tela;
- áreas de toque menores que 44 pt;
- valores fora do DS.

Não repita esses achados. Para ver antes o que ele mede, rode `explore.measure`.

A designer marca o status de cada achado e clica **Revisei a crítica** no cartão.

## 5. Gatekeeper (Fase D)

O Eureka só aceita o parecer do ds-gatekeeper (`explore.review`) depois que a designer revisou a
crítica. O pedido ao Gatekeeper leva a proposta, o lint do DS, o contexto e a crítica revisada. O
parecer é guardado na proposta, só como orientação.

## 6. O que não muda

- Registro no Notion só com o ok explícito da designer (`registro-notion.md`).
- Nada é escrito no Figma nem no DS oficial.
- O que não foi observado em produção é pergunta, não afirmação.
