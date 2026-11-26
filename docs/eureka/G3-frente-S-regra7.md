# Frente S: regra 7 do lint, componente local num arquivo de Telas e Fluxos

Branch `eureka/s-lint-componente-local` (26/11). Faz parte do E17 (G3) e vem depois da Frente R.

## O problema

O DS N Marcas — Web §1 diz:

> "Não crie componentes locais no arquivo de Telas e Fluxos"

Essa regra ficou fora do lint porque o documento não dizia que tipo de arquivo ele é. Pela resposta
4 de 24/11, o próprio arquivo declara o tipo, e o lint não tenta adivinhar.

## O que muda

- **`ds.fileType`.** Declara o tipo do arquivo: `telas-e-fluxos`, `checkout` ou `biblioteca`
  (`null` apaga a declaração). A declaração fica guardada na primeira página. Pode ser desfeita.
  Chamado sem `type`, o comando diz o que está declarado.
- **Regra `componente-local` no `ds.lint`** (só aviso). Só vale num arquivo declarado Telas e
  Fluxos. Ela aponta todo componente principal ou conjunto de variantes que nenhuma importação marcou
  como vindo de biblioteca do DS (`eureka.library`).
  - Um conjunto é apontado uma vez só, não uma vez por variante.
  - Componentes importados do DS Apps não contam.
  - Em Checkout e em biblioteca, a regra não se aplica.
  - O parâmetro `fileType` do `ds.lint` permite testar outro tipo sem mudar a declaração.

## Resultado na rodada da NV (`demo-g3s/S-componente-local.txt`)

Com o arquivo declarado Telas e Fluxos:

- **Página do quadro (8 propostas):** 0 componentes locais.
- **Página da biblioteca importada do DS Apps:** 23 conjuntos e componentes, 0 locais, porque todos
  vêm da biblioteca.
- **Componente criado na página do quadro:** é apontado: "Selo de frete: componente local num
  arquivo de Telas e Fluxos; use o componente do DS ou leve-o para a biblioteca (ou Team Component)
  pelo fluxo oficial."

## Teste

`a_local_component_is_flagged_only_in_a_file_declared_telas_e_fluxos` cobre:

- sem tipo declarado, nada é apontado;
- tipo inválido é recusado;
- componente avulso e conjunto apontados uma vez cada;
- componente de biblioteca fica de fora;
- Checkout, pela declaração e pelo parâmetro;
- desfazer.
