# E10: testes automáticos do motor (parte A)

Cobrem os pontos 1 e 2 da tabela "Falta criar" de `E10-roteiro-usabilidade.md`. O ponto 9 (compilar o TSX exportado) **não** está aqui. Código: `crates/engine/src/e10_tests.rs`. Dados só sintéticos.

> Teste automático não é validação de usabilidade. Só a parte B, com designers, diz se o produto é bom de usar.

## O que cada teste confere

### Ponto 1: editar à mão uma instância do DS sem desanexar

`hand_editing_a_library_instance_keeps_it_official_with_no_new_token_and_human_origin`

- Monta um conjunto "Button" (variantes `State=Default` e `State=Hover`, cada uma com um texto "Label" ligado à propriedade de texto "Rotulo"), marcado com `eureka.library` (o que a importação da biblioteca grava) e uma instância numa tela.
- Edita com os mesmos comandos do painel de componentes (`instance.setProperties`, sem `agent`): muda o texto e troca a variante.
- Confere: continua instância, de uma variante do **mesmo conjunto**, com a biblioteca ainda conhecida; o texto mudou; nenhuma coleção nem variável (token) nova; os componentes principais ficaram intactos; toda origem gravada na instância e nas camadas dela é `human`, e `origin.list by=ai` está vazio.

### Provas de que a checagem pode falhar

- `the_check_catches_a_detached_instance`: roda `component.detach` e a mesma função de checagem passa a reportar que não é mais instância.
- `the_check_catches_a_swap_to_another_component`: troca para um componente fora do conjunto e a checagem reporta "not in the original set".

### Ponto 2: salvar, reabrir e conferir origem e marca

`proposal_critique_review_brand_and_origin_survive_save_and_reopen`

- Quadro com duas propostas desenhadas pelo agente (`agent = Claude`), depois, como a designer (sem `agent`): `explore.choose`, `explore.critique`, `explore.critiqueReviewed`, `explore.review` (com ressalvas), `brand.apply` (modo NV nas duas telas) e uma edição manual (opacidade e texto).
- `file.saveAs` num diretório temporário, `file.open` numa sessão nova.
- Confere, por camada (as duas telas e o texto editado): `eureka.proposal`, `eureka.critique`, `eureka.review`, `eureka.brand` e `eureka.origin` idênticos antes e depois; o modo de marca de cada camada igual; origem por aspecto igual (o texto editado é `human`, a moldura do agente continua `ai`); o parecer continua o da proposta escolhida.

## Resultado e como se mostrou que pegam a regressão

Os quatro testes passam no código atual (são testes de regressão; nenhum bug do motor apareceu). Provas de que falhariam:

- Ponto 1: os dois testes "the_check_catches_*" executam a regressão (desanexar, trocar de componente) e exigem que a checagem acuse.
- Ponto 2: mutação temporária em `crates/doc/src/node.rs` (`#[serde(skip)]` em `Node::plugin_data`, ou seja, o arquivo deixa de guardar os dados do Eureka). O teste falhou em "layer 0: plugin data (proposal, critique, review, brand, origin)" e a mutação foi revertida. Não houve mutação por chave: a comparação cobre as cinco chaves e o modo de marca.

## Limites

- Só o motor. O painel, a navegação por teclado e a facilidade de uso não são testados aqui.
- O teste de salvar/reabrir leva cerca de um minuto (a crítica automática renderiza telas).
- "Origem" é informativa: o nome do agente é o que ele diz ser.
- Ponto 9 (TSX compila num projeto React) continua pendente.
