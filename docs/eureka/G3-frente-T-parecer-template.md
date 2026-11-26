# Parecer do Gatekeeper sobre a tela base do template (continuação da Frente T)

Rodado em 26/11, depois da Frente T, para conferir se a tela base do template resolvia o item 3 do
parecer da Frente Q ("galeria e bloco do produto como camadas simples").

O fluxo foi o mesmo da Frente Q, na proposta G, agora sobre a base do template. O ds-gatekeeper
respondeu em modo prévia, só leitura e sem registro no Notion
(`demo-g3v/W0-pedido-G-template.json` e `W1-parecer-G-template.json`). O parecer foi **com
ressalvas**:

- **O que está conforme na prévia:**
  - todas as instâncias vêm do DS Apps;
  - a marca NV está explícita na tela;
  - o lint voltou vazio;
  - o Action Button usa as propriedades oficiais.
- **Peças `.Nested` soltas servem para teste, não para handoff.** Elas são exclusivas do template
  e devem chegar por uma instância do template da PDP.
- **A galeria do template mostra um "Eu quero" sobre a foto**, o que dá dois botões primários na
  tela.
- **A grade de tamanhos do template é numérica (34 a 44)**, mas a G fala em "M".
- **A instância Image tem uma camada chamada "farm"**: confirmar se é só um nome interno.

## O que o template da PDP expõe (leitura do Figma, 26/11)

O conjunto `PDP` (página "Página de Produto") tem só três controles:

- **`👁️ Fixed CTA`:** liga ou desliga a área fixa.
- **`Action Button`:** liga uma área de ação com o Action Button.
- **`Produto indisponível`:** variante.

O template **não tem slot** para conteúdo.

## Consequências, para a liderança decidir

1. **A G cabe no template sem mudança estrutural.** A área de ação com dois botões já é a opção
   `Action Button` do template oficial. O que precisa de decisão é o uso dela para compra direta.
2. **As propostas que acrescentam módulos (A, B, C, D, F e H) não cabem no template.** Sem um slot
   de conteúdo, inserir um módulo exige desanexar a instância, o que é bloqueio pelo Fluxo Oficial
   §4. Para o Eureka desenhar essas propostas de forma fiel ao handoff, o template da PDP precisaria
   de um slot entre a seleção de tamanho e o fim da página. Essa é uma decisão de DS e não foi
   tomada aqui.
3. **Até lá:** a base do template (`--base=template`) serve para **exploração**, com tela parecida
   com a produção. A base desenhada continua disponível. Nenhuma das duas serve como handoff de
   propostas com módulos novos.
