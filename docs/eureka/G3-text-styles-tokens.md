# G3 — Text styles como tokens no `export.tokensCss`

Branch: `eureka/g3-text-styles-tokens`, a partir de `eureka/g3-tokens-css`. Sem merge: a junção é com a dona.

## O que mudou

O `export.tokensCss` já escrevia as variáveis da coleção (Tokens APP) como propriedades CSS, um bloco `[data-brand="…"]` por modo. Agora ele também escreve os **text styles** do arquivo como tokens, pelo nome do estilo:

```css
/* Text styles */
:root,
[data-brand] {
  --text-title-title-02-font-family: var(--font-family-brand-secondary);
  --text-title-title-02-font-weight: var(--font-weight-font-bold-weight);
  --text-title-title-02-font-size: var(--font-size-fs-lg);
  --text-title-title-02-line-height: var(--font-line-height-lh-xl);
  --text-title-title-02-letter-spacing: 0em;
}
```

- **Um token por campo:** família, peso, tamanho, altura de linha e espaçamento entre letras, com o prefixo `--text-<nome do estilo>` (mesma regra de nome do `export.tsx`: minúsculas, ASCII, `/` e espaços viram `-`).
- **Campo vinculado a variável:** sai `var(--nome-do-token)`. O valor vem do bloco da marca, então muda com o `data-brand`. Nenhum valor do DS é copiado para o estilo.
- **Peso pelo nome do estilo da fonte.** No DS Apps o peso não é uma variável numérica: o text style vincula o **estilo da fonte** a uma variável de texto (`Font/Weight/font-bold`), que guarda um nome diferente por marca (Medium, Semi bold, Book, light…). CSS precisa de número. Então o export escreve, para cada marca, `--<variável>-weight: <número>` (ex.: `--font-weight-font-bold-weight: 600` no bloco de uma marca cujo nome ali é "Semi bold"), lido do nome que a variável tem naquela marca. A conversão nome → número é a mesma que o app usa para escolher a face da fonte (`style_weight`: Light 300, Regular/Book 400, Medium 500, Semi bold 600, Bold 700…). O estilo lê `var(--font-weight-font-bold-weight)`.
- **Campo sem variável:** sai o valor do próprio estilo (em unidade CSS) e vira **aviso**, como o lint faz com valores soltos: `Body/Micro: letter-spacing 0em has no variable (the same in every brand)`.
- **Unidades:** tamanho e altura de linha em px; altura de linha em % vira número sem unidade (150% → 1.5); espaçamento em % vira `em` (−2% → −0.02em); peso sem unidade.
- **Por que `:root, [data-brand]`:** o `var()` de uma propriedade customizada é resolvido onde ela é declarada. Declarada só no `:root`, dentro de um elemento `data-brand="nv"` ela continuaria com os valores da marca padrão. Declarando também em todo `[data-brand]`, cada marca recalcula.

### Parâmetros e saída

`export.tokensCss {collections?, textStyles? (padrão true)}` → `{css, brands, textStyles, weights, warnings}`:

- `textStyles: [{style, prefix, fields: {fontFamily|fontWeight|fontSize|lineHeight|letterSpacing: {value, variable?, from?}}}]` — `from: "fontStyle"` quando o peso vem do nome do estilo da fonte.
- `weights: [{variable, token, brands: {marca: peso}}]` — os pesos por marca que o export calculou.
- `warnings` ganha: campo de text style sem variável; dois text styles com o mesmo nome CSS (o segundo fica de fora); variável de peso sem nome de estilo numa marca; token de peso que colide com o nome de uma variável.
- `textStyles: false` devolve só as variáveis, como antes.

Só leitura: nenhuma variável, estilo ou componente é alterado (os testes conferem `variable.list` e `style.list` antes e depois).

## Snapshot com text styles (fora do repo)

O snapshot antigo (`tokens-app-animale-farm-…`) tinha 2 marcas e nenhum text style. Foi gerado um novo, **lendo o Figma em modo somente leitura** (Plugin API, nada criado nem alterado), em `~/eureka-cache/tokens-app-9-marcas-textstyles-<hash>.json`:

- as 9 marcas da coleção Tokens APP (com NV);
- as 23 variáveis das demos (os valores de Farm e Animale conferem com o snapshot antigo) e as 14 de fonte que os text styles usam;
- os 13 text styles locais do DS Apps, em `meta.textStyles`, no formato da Plugin API (`fontName`, `fontSize`, `lineHeight`, `letterSpacing`, `boundVariables`).

`docs/eureka/tokens_snapshot.py` passou a ler `meta.textStyles`: o manifesto guarda os **nomes** dos estilos e das variáveis vinculadas (nenhum valor), o hash cobre os valores, e `check` acusa estilo faltando ou com vínculo diferente. Snapshots sem text styles têm o mesmo hash de antes (os dois manifestos do repo continuam `ok`). O manifesto do snapshot novo não foi versionado.

## Validação

- Testes novos (`crates/engine/src/export_tsx_tests.rs`), com coleção **sintética** de 9 marcas:
  - `text_styles_are_tokens_that_follow_the_brand`: campos vinculados viram `var()`, os tokens por marca (NV incluída), unidades, avisos dos campos soltos, linhas de `textStyles`, só leitura.
  - `text_styles_can_be_left_out_and_clashes_are_warnings`: `textStyles: false`, parâmetro inválido, nomes CSS repetidos, peso pelo nome do estilo da fonte com o número certo em cada marca (Medium 500, Semi bold 600, light 300, Book 400).
- Filtros rodados: `export_tsx`, `codegen`, `inspect`, `style_vars`, `every_command_survives_junk_params`. `cargo clippy -p figcraft-engine --all-targets -D warnings` e `cargo fmt` limpos. CI do branch: ver o commit.
- Demo real (fora do repo, tem valores do DS): snapshot novo importado num documento vazio pelo `figcraft-cli run`, text styles recriados com os mesmos vínculos, `export.tokensCss`. Resultado em `D:/Temp/claude/demo-g3-textstyles/` (`tokens.css`, `report.json`): 9 marcas (NV incluída), 13 text styles com família, tamanho e altura de linha como token, as 2 variáveis de peso do DS convertidas em número por marca, e 13 avisos, todos de espaçamento entre letras (nenhum estilo tem variável nesse campo).

## Limites

- **Text styles de biblioteca** (cópias de outro arquivo) ficam de fora, como as coleções de biblioteca: o export cobre o que é do arquivo.
- **Variável de outra coleção:** um campo vinculado a variável que não está entre as `collections` exportadas sai como `var()` de um token que esta folha não define.
- **Peso por nome:** a tabela nome → número é a do app. Nomes fora dela (ex.: "Book") caem em 400. Se a marca usar uma fonte variável com eixo `wght` próprio, o número real pode ser outro; o aviso não cobre isso.
- **Itálico, caixa (uppercase), sublinhado, espaçamento de parágrafo** não viram token ainda: só os 5 campos pedidos.
- **O `export.tsx` ainda não usa os tokens de text style:** um texto com estilo aplicado continua saindo campo a campo (tokens das variáveis ou avisos). Ligar o texto ao `--text-<estilo>-*` é o próximo passo natural.
- **Espaçamento entre letras** não tem variável em nenhum text style do DS Apps hoje: os 13 saem como aviso. É dado do DS, não falha do export.
