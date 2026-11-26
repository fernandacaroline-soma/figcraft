# G3: mapa de componente do DS para componente de código (Tarefa 3 da FILA)

Branch: `eureka/g3-code-map`, a partir do principal (`claude/eureka-design-lead-epvzr2`).

## A ideia

O Eureka já tem o **Code Connect** do FigCraft (`codeConnect.set`, `codeConnect.import` e `codeConnect.export`). Ele liga um componente (ou conjunto) ao componente de código, por linguagem:

- **caminho de importação** (`importPath`): de onde o componente é importado;
- **nome**: o nome do componente no código;
- **template**: como as propriedades do Figma viram props.

O `inspect.code` e o Dev Mode já mostram esse trecho no lugar do código gerado.

Por isso o mapa da Tarefa 3 **não é um formato novo**: é um arquivo de Code Connect (`{version: 1, links: [...]}`), guardado em `eureka-cache/`, fora do repositório. Cada entrada cobre um conjunto do DS:

```json
{"version": 1, "links": [
  {"component": "Button", "language": "react", "importPath": "@pacote/ds-apps",
   "name": "Button", "template": "<Button variant={Hierarquia} size={Tamanho} />"}
]}
```

O template usa os nomes das propriedades do Figma entre chaves:
- `{Hierarquia}` vira o valor da instância;
- `{Ícone ? "with-icon" : "plain"}` escolhe entre dois textos.

## `export.codeMap`

`export.codeMap {json | path, language?, all?}` só lê. Ele confere o mapa contra os componentes do DS no documento: componentes principais e conjuntos que uma importação marcou com `eureka.library`, como os da Frente M. Com `all: true`, confere todos os componentes.

| Resultado | O que é |
|---|---|
| `mapped` | Componente do DS com entrada válida. `unknown` são chaves do template que não são propriedades (erro de digitação). `unmapped` são propriedades que nenhuma chave lê, ou seja, uma variante que nunca chega ao código. |
| `missing` | Componente do DS no documento sem entrada no mapa. |
| `notInDocument` | Entrada para um componente que não está neste arquivo. É informação, não erro: o mapa pode cobrir a biblioteca toda. |
| `invalid` | Entrada que não ligaria: nome de código, caminho de importação, template ou linguagem inválidos. A regra é a mesma do `codeConnect.import`. |
| `counts` | Totais de cada lista. |

Depois de conferido, o mapa é aplicado com `codeConnect.import {path}`: os links ficam nos componentes do documento e o `inspect.code` passa a usá-los. A Tarefa 2 usa esses mesmos links para o `export.tsx` gerar `<Button …>` com o `import` do pacote.

## Arquivo-modelo

Criei `~/eureka-cache/ds-apps.connect.json` com os quatro conjuntos da Frente M: Button, Selector Size, Choice Tag e Bottom Sheet.

- **Os nomes de código e o pacote (`@azzas/ds-apps`) são propostas.** Quem confirma é a Tecnologia; não achei o pacote real nas fontes.
- **Os templates estão vazios (`<Button />`).** Por isso o `export.codeMap` vai listar todas as propriedades como `unmapped` até alguém preencher as props.
- **O snapshot de componentes da Frente M não está neste computador.** Por isso não rodei o mapa contra os conjuntos reais. Rodar `export.codeMap {path}` com o arquivo importado é o primeiro passo no outro computador.

## Validação

Os testes ficam em `crates/engine/src/code_map_tests.rs` e usam componentes sintéticos:

- `the_map_is_checked_against_the_design_systems_components` cobre:
  - `unknown` e `unmapped`;
  - entrada inválida (nome de código começando com número) e linguagem inválida;
  - `notInDocument`;
  - um componente local que não conta como DS;
  - a confirmação de que nada foi ligado;
  - outra linguagem lendo as próprias entradas.
- `a_design_system_component_with_no_entry_is_missing` cobre `missing`, a lista vazia de `unmapped` quando o template lê tudo, e o `all`.
- `the_map_comes_from_a_file_or_text_and_junk_is_refused` confirma que o arquivo e o texto dão o mesmo resultado e que parâmetros inválidos são recusados.

Também rodei o `clippy -D warnings` no engine; o resultado do CI está no commit.

## Limites

- Componente do DS é o que tem `eureka.library`. Um componente de biblioteca importado sem essa marca só aparece com `all: true`.
- O nome no mapa é comparado com o nome do conjunto no documento, de forma exata. Um conjunto renomeado no Figma aparece como `missing` mais `notInDocument`.
- Não há verificação de que o pacote ou o componente existem no código: isso é do lado da Tecnologia.
