# Frente V: contraste em todas as marcas

Branch `eureka/v-contraste-marcas` (26/11). Faz parte do E17 (G3).

## O problema

O DS Apps é multimarca: a mesma tela pode ser publicada em Animale, Farm, NV e mais 6 marcas. A
medição de contraste da crítica (Frente O) olhava só a marca da tela. Um texto podia passar na NV e
falhar na Cris Barros sem ninguém ver.

## O que muda

- **`allBrands`** em `explore.measure` e em `explore.critique`. Mede o contraste de cada texto em
  todos os modos da coleção de marca (Tokens APP, ou Tokens WEB no Web) e gera um achado por
  texto, com as marcas em que ele falha e a razão de contraste de cada uma. Por exemplo: "fica
  abaixo do contraste AA em 1 de 9 marca(s): NV (4.1:1)".
- **O achado diz de onde vem o texto:** "Vem do componente X do DS." Assim a designer sabe que o
  problema não é da proposta.
- **O que deixou de ser medido:**
  - **Texto de componente desabilitado.** O WCAG 1.4.3 isenta componentes inativos.
  - **Texto da mesma cor do fundo medido.** Isso só acontece quando ele está sobre uma foto ou um
    gradiente, que não dá para medir. Antes saía um falso 1,0:1.
- **Correção de um bug.** A cor ligada a um token fica na chave `fill`, e a medição só lia
  `fills/0/color`. Por isso usava a cor já resolvida na marca da tela, e as outras marcas não
  tinham como ser medidas.

## Resultado na rodada da NV (base do template, `demo-g3v/V-contraste.txt`)

**Todos os achados vêm de componentes do DS, nenhum das propostas.** Fica para a liderança do DS
avaliar; nada foi alterado.

- **`.Nested / pdp-main-info` e `.Nested / fixed-component`** (template oficial da PDP): os textos
  secundários ("10x R$ 159,90 sem juros", "Ref …", o preço e "em até 12x") ficam entre **2,7 e
  3,6:1 em 7 das 9 marcas**. O AA pede 4,5:1. Passam só em Cris Barros e Hering. Aparecem em todas
  as propostas, porque fazem parte da tela base.
- **Bottom Sheet:** o subtítulo fica com 4,1:1 na NV.
- **Snackbar de informação:** 2,5:1 na Cris Barros.

## Testes

- **`contrast_is_measured_in_every_brand_when_asked`:** a tela está em Farm e passa. Com
  `allBrands`, o texto falha só na NV, "1 de 2".
- **`a_disabled_text_and_a_text_over_an_unmeasured_background_are_not_flagged`:** o texto de
  "Desabilitado=On" é isento. A mesma variante habilitada é medida.
- **`a_low_contrast_text_inside_a_ds_component_names_it`:** o achado nomeia o componente.

## Rodar

```
python3 docs/eureka/contraste_marcas.py W <ids das telas>
```
