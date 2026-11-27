# E10: o TSX exportado compila?

**Resposta curta: não, como sai do `export.code`. Compila depois de separar o CSS** (que o export deixa dentro do arquivo `.tsx`).

## Como foi medido

Script: `docs/eureka/e10_tsx_compila.py` (`python -X utf8`; fora do CI).

1. `figcraft-cli run` constrói um documento sintético (tela com Button ligado por Code Connect, estilo de texto, tokens ligados a camadas) e chama `export.code` (que junta `export.tsx` e `export.tokensCss`).
2. Cria um projeto temporário React + TypeScript (`react`, `@types/react`, `typescript` como devDependencies locais; `jsx: react-jsx`, `strict`), com `declare module` falsos para cada pacote importado pelo TSX (cada export nomeado vira um componente com props soltas) e `declare module "*.css"`.
3. `npm install` local e `npx --no-install tsc --noEmit`, duas vezes: **(a)** o arquivo como o export devolve; **(b)** com o bloco de CSS separado do TSX.

Uso: `python -X utf8 docs/eureka/e10_tsx_compila.py --cli <figcraft-cli> [--out relatorio.json] [--keep]`. Para um arquivo real: `--input ARQUIVO.figcraft --ids ID[,ID] [--map MAPA.json]` (o relatório de arquivo real só traz códigos e mensagens de erro, nunca o código-fonte, a menos que se passe `--quote`).

## Resultado (documento sintético)

| Variante | Compila | Erros |
|---|---|---|
| (a) como exportado (`PDPMobile.tsx`) | **não** | 29, todos de sintaxe: TS1128, TS1005, TS1134, TS1351 |
| (b) CSS separado do TSX | **sim** | 0 |

Importações do TSX: `import { Button } from "@acme/ds-apps";` e `import "./pdp-mobile.css";`. Nenhum token usado ficou sem definição em `tokens.css`. O export avisou de 4 valores sem token (fill, família, tamanho e peso do texto do documento sintético), que são avisos de dívida do DS e não erros de compilação.

## Causa raiz dos erros (em `export.tsx`)

1. **O CSS vem dentro do mesmo texto do TSX.** `export.tsx` devolve em `code` o componente, uma linha `/* pdp-mobile.css */` e as regras (`.pdp-mobile { ... }`). `export.code` grava esse texto inteiro como `<Nome>.tsx`. O TypeScript lê `.pdp-mobile {` como código: TS1128/TS1005 nas linhas de seletor e TS1351 em valores como `375px` (número seguido de identificador). Os 29 erros são todos dessa mesma causa.
2. **O arquivo `pdp-mobile.css` que o TSX importa não é gravado.** O TSX tem `import "./pdp-mobile.css"`, mas `export.code` só grava `<Nome>.tsx` e `tokens.css`. Mesmo separando o CSS à mão, quem recebe precisa criar o arquivo com o bloco embutido (o script faz isso na variante b). Em um bundler isso seria erro de módulo não encontrado, que o `tsc` não mostra porque a declaração `*.css` o cobre.
3. **Nada mais falhou:** o componente (JSX, `data-brand`), a chamada `<Button variant="Secundário" />` e o import do pacote fecham com tipos soltos. A conferência contra os tipos reais do `@azzas/ds-apps` (nomes e tipos das props) **não foi feita**: os tipos do script são falsos e aceitam qualquer prop.

## Proposta (não aplicada: `export.tsx` não foi alterado nesta tarefa)

- `export.code` deveria gravar o CSS do componente em um arquivo próprio (`<nome>.css`, o nome que o TSX já importa) e deixar no `.tsx` só o TSX; ou o `import` e o bloco embutido sairiam do `.tsx` e o CSS iria em `tokens.css`. `export.tsx` pode manter o bloco em `code` se os testes atuais dependem disso (eles conferem o `code` inteiro): a separação cabe em `code_files` (`crates/engine/src/dev/export_tsx.rs`).
- Depois da correção, rodar este script de novo: a variante (a) deve passar a compilar.

## Limites

- Arquivo real (proposta G, base do template, mapa ligado): **não rodado**. Em `~/eureka-cache` há só os snapshots de tokens e o mapa `ds-apps.connect.json`, sem o documento `.figcraft` da tela, então só o caminho sintético foi exercitado. O parâmetro `--input/--ids/--map` está pronto para quando o arquivo existir.
- Tipos do DS falsos (por pedido): o resultado diz que a sintaxe e as importações fecham, não que as props batem com o pacote real.

## Correção (PC2, 26/11): o CSS sai em arquivo próprio

Branch `eureka/e10-css-separado`. O `export.code` (e o `explore.exportCode`, que usa o mesmo `code_files`) agora separa o código do `export.tsx` em arquivos:

- `<Nome>.tsx`, só com o TSX;
- `tokens.css`, a folha de tokens, como antes, na segunda posição;
- um `<classe>.css` por componente, o arquivo que o TSX já importava (`import "./pdp-mobile.css"`), com as regras do bloco `/* pdp-mobile.css */`.

O `code` do `export.tsx` continua igual. Os testes de ida e volta e o spike dependem dele.

Rodei o script de novo com a CLI corrigida, no documento sintético: **(a) como exportado: compila, 0 erros** (antes eram 29); (b) também 0.

Testes novos: `two_screens_export_one_tsx_with_no_css_and_a_stylesheet_each` e mais asserções em `export_code_writes_both_files_and_says_what_is_a_token` e no teste do `explore.exportCode`. Os 3 falham sem a correção. A proposta G real continua sem rodar, porque não há `.figcraft` dela neste PC.
