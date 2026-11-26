# Spike: TSX → Eureka → TSX (Tarefa 6 da FILA)

Branch: `eureka/g3-spike-tsx-volta`, a partir do principal. É só o spike: um script e este relatório. O motor não mudou.

## O que foi feito

`docs/eureka/spike_tsx_volta.py` lê um TSX como o que o `export.tsx` gera e o reconstrói como camadas no FigCraft, usando comandos (`figcraft-cli run`). Depois roda o `export.tsx` de novo e compara as duas versões linha a linha.

- **Entrada.** O TSX é JSX de `<div>`/`<p>` com `className`, mais o CSS no fim do arquivo. As variáveis vêm de um snapshot do DS, fora do repositório.
- **Leitura do CSS.**
  - `display: flex` e `flex-direction` viram layout automático vertical ou horizontal.
  - `gap` e `padding` viram espaçamento.
  - `justify-content` e `align-items` viram alinhamento.
  - `align-self: stretch` vira preenchimento no eixo cruzado.
  - Elemento sem `height` passa a abraçar o conteúdo.
  - `position: absolute` com `left` e `top` vira posição fixa.
  - `background`, `color`, `border-radius`, `font-size` e `font-family` viram as propriedades correspondentes.
- **Tokens.** Um valor `var(--token)` vira **vínculo de variável**: o script procura a variável cujo nome dá aquela variável CSS, pela mesma regra do `ds_var_name`. Um valor fixo vira valor fixo.
- **Marca.** `data-brand` vira o modo da coleção cujo modo tem aquele nome.
- **Perdas.** Propriedades que a volta não lê, e tokens que o snapshot não tem, entram numa lista de perdas no relatório.

## Resultado na tela PDP da demo

Usei `demo-g3/export-pdp-mobile.tsx`: 7 camadas com auto layout, 14 tokens e a marca Animale. O snapshot foi o de 2 marcas do cache.

| | Linhas | Iguais ao original |
|---|---|---|
| Ida e volta (TSX → camadas → TSX) | 77 | **76** |
| Segunda volta (sobre a primeira) | 77 | **77** (estável) |

- Foram 38 comandos, sem erro: 22 vínculos de variável, a marca e o layout.
- A **única diferença** é o nome da função: `PDPMobile` virou `PdpMobile`. O TSX guarda a classe (`pdp-mobile`), não o nome da camada ("PDP Mobile").
- Os 14 tokens voltaram como vínculos. O CSS, o layout, a marca e os avisos de valor solto (família e peso) saíram idênticos.

## O que se perde na volta

**Por limite do TSX** (a informação não está no arquivo):

- **Nomes das camadas.** Só a classe volta, então maiúsculas, espaços e acentos se perdem ("Preço" vira `preco`). Isso muda o nome da função e o nome das camadas nos avisos do cabeçalho.
- **Componentes.** Uma instância desenhada (inline) volta como frames soltos. Uma chamada ao componente de código (`<Button …/>`, da Tarefa 2) não é lida. O vínculo com a biblioteca do DS (`eureka.library`) e as variantes se perdem.
- **Estilos de texto.** O `export.tsx` ainda escreve campo a campo, sem `--text-<estilo>-*`. Volta só o que está no CSS da camada.
- **Tudo o que o export não escreve:** camadas ocultas, protótipo, anotações, `pluginData` (origem da proposta, crítica e parecer) e as outras coleções de modo.

**Por limite do spike** (dá para ler, mas o script não lê ainda):

- **Bordas, sombras, opacidade, rotação, imagens e SVG.** Viram "propriedade que a volta não lê".
- **`border-radius` com quatro valores**, um por canto.
- **Peso da fonte**, que volta como a face padrão da família.

## Conclusão

Para telas montadas com auto layout e tokens, como as do gerador de propostas, a ida e volta pelo TSX preserva o layout, os tokens e a marca. A volta estabiliza: a segunda passada não muda nada.

O que se perde é o que o TSX nunca carregou: nomes, componentes, estilos, estados e metadados do Eureka. Por isso o TSX não serve como fonte do design. Isso confirma a ADR-003: o documento é a fonte, e o TSX é saída gerada.

Um importador de verdade teria de ler:
- as chamadas de componente, voltando à instância pelo mapa da Tarefa 3, ao contrário;
- um comentário de metadados que o export escrevesse (nomes das camadas e ids).

## Como rodar

```
python -X utf8 docs/eureka/spike_tsx_volta.py --tsx <arquivo.tsx> \
  --snapshot ~/eureka-cache/tokens-app-animale-farm-<hash>.json \
  --cli <figcraft-cli> --out <pasta fora do repo>
```

O script grava em `--out`: `script.json`, `run.json`, `volta.tsx` e `relatorio.json`, com as diferenças e as perdas.
