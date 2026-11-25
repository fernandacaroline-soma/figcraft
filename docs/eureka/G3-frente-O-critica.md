# Frente O — crítica de design entre a escolha e o Gatekeeper

Branch `eureka/o-critica` (25/11). Faz parte do E17 (G3). Segue a Frente N.

## O problema

Na skill gerador-propostas (v0.3.0), a ordem é: propostas → **a designer escolhe → crítica de
design (Fase C, `references/critica.md`) → a designer revisa a crítica → ds-gatekeeper (Fase D)**.
No Eureka, depois do botão "Escolher esta proposta", a próxima etapa já era o parecer do
Gatekeeper. A crítica não existia.

## O que muda

### 1. `explore.critique`: a crítica fica guardada na proposta escolhida

Cada achado tem os campos do roteiro da skill:

| Campo | Conteúdo |
|---|---|
| `area` | fluxo, estados, falhas, casos de borda, conteúdo e interação, acessibilidade, viabilidade, medição |
| `scenario` | onde acontece |
| `consequence` | o que a usuária sofre |
| `severity` | crítica, alta, média, baixa |
| `evidence` | a fonte, ou `inferência` quando ninguém observou |
| `recommendation` | o que mudar |
| `status` | aberto, resolvido, decisão da designer (começa como aberto) |

A crítica também guarda a proposta refinada (`refined`) e as decisões em aberto (`open`).

- Só a proposta marcada como escolhida pode ser criticada.
- Área, severidade ou status fora da lista são recusados, e o erro mostra as opções válidas.
- Os achados são ordenados do mais grave para o menos grave e numerados.
- Uma crítica nova substitui a anterior e apaga o parecer dado sobre ela.
- Escolher outra proposta apaga a crítica da anterior, porque a crítica era daquela escolha.
- Tudo pode ser desfeito.

### 2. O que o Eureka mede sozinho (`explore.measure`)

Esses achados entram na crítica com a evidência "Eureka (medido)":

- **Estados:** variantes de estado dos componentes do DS usados na tela (Desabilitado, Erro,
  Loading, Vazio e outros) que nenhuma instância da tela mostra.
- **Contraste de texto:** WCAG 2.1 AA, critério 1.4.3. O cálculo usa as cores que os tokens assumem
  **na marca da tela**. O mesmo token pode passar na Farm e falhar na NV, e o teste cobre esse
  caso.
- **Área de toque:** instâncias soltas na tela menores que 44 × 44 pt, a referência das Apple
  Human Interface Guidelines. Sai um achado por componente e tamanho.
- **Lint do DS:** valores da tela fora das variables e estilos do DS.

O resto da crítica (fluxo, falhas, casos de borda, conteúdo, viabilidade, medição) vem do revisor,
o Claude seguindo o roteiro da skill. O que não foi observado em produção entra como pergunta, com
a evidência `inferência`.

### 3. Cartão: Escolhida → Crítica → Parecer

- A crítica aparece com a contagem por severidade e cada achado completo.
- Cada achado tem um seletor de status. Mudar o status roda `explore.critiqueStatus`.
- O botão **"Revisei a crítica"** (`explore.critiqueReviewed`) libera o Gatekeeper. Depois do
  clique, aparece "Ainda sem parecer." até o parecer chegar.

### 4. Trava de ordem

- `explore.review` (o parecer do Gatekeeper) recusa enquanto a crítica não foi revisada pela
  designer.
- `gatekeeper_bridge.py export-choice` também recusa nessa situação. Quando a crítica foi
  revisada, o pedido ao Gatekeeper leva a crítica junto (`design_critique`).

## Rodada NV 01, proposta A (`demo-g3o/`)

Rodei `rodada_n.py` (o quadro da Frente N) e depois `rodada_o.py` na proposta A, "Tamanho na hora
de escolher". A crítica do roteiro está em `rodada_nv01_critica_A.json`.

Resultado (`O-critica.txt`): **7 achados**, sendo 1 alta, 4 médias e 2 baixas.

**Medidos pelo Eureka (2):**
- o Selector Size aparece só com `Desabilitado=Off`, e a tela não mostra o estado desabilitado;
- os 5 seletores de tamanho medem 40 × 40 pt.

**Do roteiro (5):**
- produto sem modelo ou sem as medidas cadastradas (alta);
- o resumo promete as medidas da peça, mas a tela só mostra as medidas da modelo;
- o que acontece ao selecionar outro tamanho;
- os dados existem no catálogo hoje? (pergunta para a engenharia);
- métrica e teste A/B.

**Ordem:** antes da revisão da crítica, o Gatekeeper foi **recusado**, como manda a skill.

**Arquivos:**
- **O1:** o cartão com a crítica.
- **O2:** o cartão depois de um status e de "Revisei a crítica".
- **O0:** o que foi medido.

O status e a revisão foram **um teste do cartão**, feito num arquivo local. A revisão real da
crítica e a escolha da rodada são da designer. Nada foi gravado no Notion nem no Figma, e nenhum
componente, token ou regra do DS mudou.

## Limites

- **O snapshot do DS Apps é da rodada.** Ele não traz todas as variantes de cada componente. O
  Button, por exemplo, só vem com `Desabilitado=Off` e `Loading=False`, então nenhum estado do
  Button pôde ser apontado. Com um snapshot maior (`ds_components_snapshot.js`), a medição de
  estados fica completa.
- **Contraste:** usa o fundo do contêiner preenchido mais próximo. Não mede texto sobre imagem nem
  sobre um irmão sobreposto, e ignora fundos translúcidos.
- **Área de toque:** o Eureka não sabe se a instância é tocável, por isso o achado diz "se for
  tocável" e tem severidade baixa. Instâncias dentro de outras instâncias ficam de fora, porque são
  do componente.
- **Lado da skill:** a skill ainda não chama `explore.critique` sozinha. Essa é a mesma referência
  pendente da Frente N (`references/eureka.md`), a combinar com a dona da skill.

## Testes

- `the_eureka_measures_missing_states_contrast_in_the_brand_and_small_touch_targets`: estado
  faltando, contraste que falha na NV e passa na Farm com o mesmo token, e um alvo pequeno (Tag
  80 × 24 pt) sem acusar o Button de 160 × 48 pt.
- `the_critique_comes_between_the_choice_and_the_gatekeeper`: só a escolhida; recusas por área,
  severidade, status e número; ordem por severidade; trava do Gatekeeper; desfazer; crítica nova
  apaga o parecer; outra escolha apaga a crítica.
- `a_verdict_is_kept_on_the_chosen_proposal_only` (atualizado): agora passa pela crítica revisada.

## Rodar

```
python3 docs/eureka/rodada_n.py WORKDIR eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-nv-round-bd88e36339f2.json docs/eureka/rodada_nv01_propostas.json
python3 docs/eureka/rodada_o.py WORKDIR <id da tela A> docs/eureka/rodada_nv01_critica_A.json
```
