# Frente T: a tela base da PDP com as peças do template oficial

Branch `eureka/t-pdp-template` (26/11). Faz parte do E17 (G3).

## O problema

O Gatekeeper apontou um problema na Frente Q: a galeria, o nome e o preço da tela base eram
camadas simples, e não componentes do DS. A tela base da PDP era desenhada à mão. O DS Apps tem um
template oficial da PDP (página "Página de Produto"), montado com peças próprias.

## O que muda

### 1. Snapshot com as peças do template

O `ds_components_snapshot.js` ganhou mais componentes de origem:

- `.Nested / PDP-product-gallery`: só a variante sem "Complete o look", porque a outra traz cards
  de produto.
- `.Nested / pdp-main-info`: as três variantes.
- `.Nested / size-selection-PDP`: com e sem estoque.
- `.Nested / fixed-component`: a variante sem promoção.
- `Image`: só as fotos de produto.

O snapshot (`eureka-cache/components-app-frente-t-da860ee1caab.json`, cerca de 1 MB) foi buscado
**por um subagente**. As 5 partes foram extraídas do histórico dele, sem passar pela sessão
principal e sem cópia manual. O Figma foi só lido.

### 2. `rodada_n.py --base=template`

Com essa opção, a tela base da PDP é montada com as peças do template:

1. galeria;
2. módulos que a proposta põe antes das informações;
3. informações do produto;
4. seleção de tamanho;
5. módulos da proposta;
6. área fixa na base.

É uma página longa, com rolagem, como na produção. Os textos dentro das peças são o conteúdo de
exemplo do template. Sem a opção, a tela base continua a desenhada.

### 3. Nome de token em mais de uma coleção

As peças do template trazem uma cópia remota de alguns tokens, numa coleção chamada "Tokens", com
os mesmos nomes da Tokens APP. Quando a descrição da tela usa um nome que existe em mais de uma
coleção, agora vale o da **coleção oficial** (Tokens APP, ou Tokens WEB no Web). Um nome
desconhecido continua sendo recusado. Teste:
`a_token_name_in_two_collections_means_the_official_one`.

### 4. Lint: o vínculo que o componente já tem é herdado

O lint já tratava assim os valores fixos: o que o componente principal já tem é do DS, conta como
herdado e não é apontado na tela. Agora vale o mesmo para as regras de vínculo (coleção não oficial
e escopo). Sem isso, a rodada com o template dava 32 achados que não eram das propostas. Teste:
`a_binding_the_main_component_has_is_inherited_not_the_screens`.

## Resultado (`demo-g3t/`)

As 8 propostas foram desenhadas sobre o template (`T1`), com **lint 0 nas propostas** e 219
valores herdados dos componentes do DS. As fotos aparecem hachuradas porque o snapshot traz a
referência da imagem do DS, não a imagem.

## Achado sobre o DS (para a liderança decidir; nada foi alterado)

No `.Nested / pdp-main-info` do template oficial da PDP, o texto **"Nome do produto"** usa 4 tokens
de tipografia de uma coleção remota **"Tokens"**, e não da Tokens APP:

- `Font/Family/Brand-Primary`
- `Font/Weight/font-regular`
- `Font/Line Height/lh-md`
- `Font/Size/fs-xxs`

Pelo Fluxo Oficial §10, os tokens precisam vir das coleções corretas. O lint aponta isso quando roda
sobre o componente principal. Sobre as telas, ele não aponta, porque o problema é do DS.

## Limites

- **Textos do template:** não dá para trocar, porque não são propriedades do componente. A PDP
  mostra o produto de exemplo do template ("Vestido De Tricot Midi…").
- **`picked`, `button` e `no_button`:** na base do template, o tamanho selecionado e o texto do
  botão "Eu quero" não mudam. `no_button` esconde a área fixa inteira.
- **Fotos:** aparecem como hachurado.
