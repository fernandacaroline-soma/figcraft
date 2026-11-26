# Frente Z: o cartão liga o mapa de código

Branch `eureka/z-mapa-no-cartao` (26/11).

## Problema

Na Frente Y, o código da G saiu com os componentes do DS só porque o mapa foi ligado antes, por
comando (`codeConnect.import`). Quem clicasse em "Exportar a proposta como código" no cartão recebia
os componentes do DS desenhados como camadas, com os 56 valores sem token, e sem saber por quê.

## O que muda

### Comando `explore.codeMap`

- **Sem parâmetros:** diz qual mapa o arquivo ligou e quantos componentes do DS há no arquivo, quantos
  estão ligados e quais não têm componente de código.
- **Com um mapa** (`path`; ou `data` em base64 e `name`, como o navegador entrega o arquivo; ou `json`):
  1. confere o mapa (`export.codeMap`);
  2. liga os componentes (`codeConnect.import`);
  3. guarda no arquivo o nome do mapa e as contagens, na primeira página (`eureka.codeMap`).
- **Desfazer:** uma única ação desfaz tudo, os vínculos e o nome guardado.
- **Mapa inválido** (não é JSON, versão diferente de 1, parâmetro errado): é recusado, e nada fica
  guardado.

Os vínculos ficam salvos no arquivo. Por isso, o mapa é ligado uma vez por arquivo, e não a cada
exportação.

### No cartão, abaixo de "Exportar a proposta como código"

- **Sem mapa:** "Sem mapa de código: os componentes do DS saem desenhados como camadas." e o botão
  **"Ligar um mapa de código…"**, que abre o seletor de arquivo (também no navegador). Captura:
  `demo-g3z/Z0-sem-mapa.png`.
- **Com mapa:** "Mapa de código: ds-apps-pdp.connect.json · 36 componentes do DS ligados", um aviso
  quando algum componente ficou sem entrada e o botão "Ligar outro mapa de código…". Captura:
  `demo-g3z/Z1-com-mapa.png`.
- **Depois de exportar:** além dos valores sem token, o aviso diz quantos componentes do DS saíram
  desenhados por falta de componente de código.

## Teste no app (G, base do template, NV)

1. A rodada foi montada com `rodada_n.py --base=template`. Depois vieram a escolha e a crítica revisada
   (`rodada_o.py`).
2. O parecer foi o salvo na Frente Q (`demo-g3q/Q2-parecer-gatekeeper.json`), usado só para levar o
   cartão até o botão de exportar. Ele não é um parecer novo.
3. Antes de ligar o mapa, `explore.codeMap` dizia: 36 componentes do DS, 0 ligados, 36 sem entrada.
4. Ligado o `ds-apps-pdp.connect.json` da Frente Y: 36 ligados, 0 sem entrada, 0 inválidos.
5. `explore.exportCode` gravou o `.tsx` e o `tokens.css`, com 0 valores sem token. A única nota é a do
   Action Button (os dois rótulos que a chamada não leva).
6. Desfazer voltou ao estado sem mapa.

## Validação

- `the_code_map_is_linked_once_and_the_file_remembers_it`:
  - antes, sem mapa e sem `import`;
  - depois do arquivo em base64, o mapa guardado, os vínculos e o `import` no export, sem outro passo;
  - o desfazer volta tudo ao estado anterior.
- `a_bad_code_map_is_refused_and_nothing_is_kept`: cinco entradas inválidas são recusadas, e nada fica
  guardado.
- `cargo clippy -D warnings` no motor e na interface: limpo. Testes de idioma: passam.
- Neste Mac também falham, sem esta mudança: 3 testes do mapa mental no motor e 1 de área de
  transferência na interface (`clipboard_fence_tests`). No CI, eles passam.

Figma só lido. Nenhum componente, token ou regra do DS foi alterado.
