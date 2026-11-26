# Pendências que esperam decisão (26/11)

Cada item abaixo depende de você ou de outra pessoa. Nada disto foi feito: nenhum token, componente
ou regra do DS foi alterado, o Figma só foi lido e ninguém foi contatado.

## Resumo

| # | Pendência | Quem decide | Recomendação |
|---|---|---|---|
| 1 | Contraste baixo em textos do template da PDP e de 2 componentes | Liderança do DS Apps | Levar os 3 achados como um só pedido de revisão de tokens de cor |
| 2 | "Nome do produto" usa tokens de uma coleção remota "Tokens" | Liderança do DS Apps | Pedir a troca para a Tokens APP no componente |
| 3 | Template da PDP sem slot de conteúdo | Liderança do DS Apps | Propor um slot entre a seleção de tamanho e o fim da página |
| 4 | Action Button sem os rótulos dos botões como propriedade | Liderança do DS Apps (com a Tecnologia) | Por ora, seguir com a nota para o dev; propor a propriedade junto com o item 3 |
| 5 | Referência da skill para a Naty | Você (e depois a Naty) | **Rascunho atualizado (26/11):** revisar e enviar |
| 6 | Nomes de código e pacote do mapa | Tecnologia | Enviar o texto pronto abaixo |
| 7 | Atualização do plugin DS | Você | Atualizar no fim de uma sessão, não no meio de uma frente |
| 8 | Branch `eureka/t-pdp-template` com imagem de 6 MB | Você | Apagar |

---

## 1. Contraste baixo no template da PDP e em 2 componentes

**O que é.** O Eureka mediu o contraste dos textos nas 9 marcas do Tokens APP. Os achados vêm de
componentes do DS, não das propostas:

- **Template da PDP** (`pdp-main-info` e `fixed-component`): o parcelamento ("10x R$ 159,90 sem
  juros"), a referência ("Ref …"), o preço e "em até 12x" ficam entre **2,7 e 3,6:1 em 7 das 9
  marcas**. O mínimo AA é 4,5:1. Só Cris Barros e Hering passam.
- **Bottom Sheet:** o subtítulo fica com **4,1:1 na NV**.
- **Snackbar de informação:** **2,5:1 na Cris Barros**.

Fonte: `G3-frente-V-contraste-marcas.md`.

**Quem decide.** A liderança do DS Apps.

| Opção | A favor | Contra |
|---|---|---|
| A. Levar os 3 como um só pedido de revisão de cor | Um único pedido; o template aparece em todas as telas de PDP | Mexe em token de cor compartilhado: precisa de revisão por marca |
| B. Levar só o template da PDP agora | É o de maior impacto (7 marcas, toda PDP) | Bottom Sheet e Snackbar ficam esquecidos |
| C. Não levar | Nenhum custo agora | Texto abaixo do AA continua em produção |

**Limite.** A medição foi feita sobre o snapshot do DS no Eureka, não no app publicado. Antes de
corrigir, vale conferir uma tela real por marca (não verificado em produção).

**Recomendação.** A.

**Depois do "sim".** Eu monto a lista por componente, texto, marca e razão de contraste, com a fonte
da regra, para você levar. Não altero token nenhum.

---

## 2. "Nome do produto" com tokens de uma coleção remota

**O que é.** No `.Nested / pdp-main-info` do template oficial da PDP, o texto "Nome do produto" usa 4
tokens de tipografia de uma coleção remota chamada **"Tokens"**, e não da Tokens APP:
`Font/Family/Brand-Primary`, `Font/Weight/font-regular`, `Font/Line Height/lh-md` e
`Font/Size/fs-xxs`. O Fluxo Oficial §10 pede os tokens da coleção correta.

Fonte: `G3-frente-T-pdp-template.md`.

**Quem decide.** A liderança do DS Apps.

| Opção | A favor | Contra |
|---|---|---|
| A. Pedir a troca para a Tokens APP no componente | Corrige a origem; a troca de marca passa a valer nesse texto | Mexe num componente oficial: precisa de branch e revisão do DS |
| B. Registrar como exceção conhecida | Nenhuma mudança agora | O texto pode não mudar de marca como os outros (não verificado) |

**Recomendação.** A.

**Depois do "sim".** Eu preparo a descrição para o pedido (componente, camada, os 4 tokens e a regra).
A mudança em si é do DS.

---

## 3. Template da PDP sem slot de conteúdo

**O que é.** O template da PDP só tem 3 controles: `👁️ Fixed CTA`, `Action Button` e
`Produto indisponível`. Não tem slot de conteúdo. As propostas que acrescentam módulos (A, B, C, D, F
e H) não cabem nele sem desanexar a instância, o que o Fluxo Oficial §4 bloqueia. A G cabe sem
mudança, porque a área com dois botões já é a opção `Action Button`.

Fonte: `G3-frente-T-parecer-template.md`.

**Quem decide.** A liderança do DS Apps.

| Opção | A favor | Contra |
|---|---|---|
| A. Propor um slot entre a seleção de tamanho e o fim da página | As propostas com módulo novo viram handoff sem desanexar | Muda um template compartilhado por todas as marcas |
| B. Manter sem slot | Template estável | Proposta com módulo novo só serve para exploração |

**Recomendação.** A, como proposta para a liderança avaliar. Não é uma mudança que a gente faz.

**Depois do "sim".** Eu escrevo a proposta do slot (onde fica, o que aceita, o que não muda) para você
levar. Enquanto isso, a base do template continua só para exploração.

---

## 4. Action Button não expõe os rótulos dos botões

**O que é.** Na G, os dois botões da área de ação têm os textos "Comprar agora · M" e "Adicionar à
sacola". O Action Button não tem esses textos como propriedade. Por isso, no código exportado, a
chamada `<ActionButton …/>` não leva os textos. Desde a Frente Y, eles vão como nota para o dev no
topo do arquivo.

Fonte: `G3-frente-Y-mapa-template.md`.

**Quem decide.** A liderança do DS Apps, com a Tecnologia (o componente de código precisa receber os
textos).

| Opção | A favor | Contra |
|---|---|---|
| A. Seguir com a nota para o dev | Já funciona; nada muda no DS | O dev copia os textos da nota à mão |
| B. Propor propriedades de texto no Action Button | O código sai completo | Propriedade nova num componente oficial, no Figma e no código |

**Recomendação.** A por ora. Levar B junto com o item 3, porque os dois mexem na PDP.

**Depois do "sim".** Para A, nada a fazer. Para B, eu escrevo a proposta da propriedade.

---

## 5. Referência da skill para a Naty

**O que é.** O rascunho `rascunho-skill-references-eureka.md` propõe um arquivo `references/eureka.md`
para a skill gerador-propostas, explicando como desenhar as propostas no Eureka.

**Quem decide.** Você decide se e quando envia. A Naty decide se entra na skill.

**Revisão do rascunho.** O que está escrito está certo, mas ele é de antes das frentes de hoje e
**não está pronto para enviar**. Faltam:

1. **Base do template oficial** (Frente T): a tela base pode partir das peças do template da PDP, e
   isso só serve para exploração (item 3).
2. **Contraste em todas as marcas** (Frente V): a seção 4 diz "contraste do texto na marca da tela".
   Hoje o Eureka mede nas 9 marcas e diz de qual componente do DS vem o texto.
3. **Aprovar o layout e a memória visual** (Frente U), que é a Fase E da skill: o botão "Aprovar o
   layout" e o rascunho da entrada no modelo da memória visual. A entrada não é gravada no Notion sem
   o ok da designer.
4. **Exportar como código** (Frentes X, Y e Z): o botão do cartão, o mapa de código do DS e as notas
   para o dev.


| Opção | A favor | Contra |
|---|---|---|
| A. Atualizar o rascunho e depois enviar | Ela recebe o fluxo completo de uma vez | Mais uma rodada antes de enviar |
| B. Enviar como está | Sai hoje | Ela recebe um fluxo incompleto, e vai precisar de uma segunda versão |

**Recomendação.** A.

**Feito (26/11).** O rascunho ganhou os 4 pontos: a tela base do template na seção 2, o contraste
em todas as marcas na seção 4, a seção 6 (aprovação e memória visual, Fase E) e a seção 7 (código
para os devs). Conferi no repositório dela, só lendo: `figma-ds.md`, `critica.md`,
`registro-notion.md`, `memoria-visual.md` e `ds-gatekeeper.md` existem com esses nomes. Falta só
a sua revisão antes de enviar. Não enviei nada e não mexi no repositório dela.

---

## 6. Nomes de código e pacote do mapa (Frente Y)

**O que é.** O mapa de código liga 36 componentes do DS Apps aos componentes de código. Os nomes
(`PdpProductGallery`, `PdpMainInfo`, `SizeSelectionPdp`, `ActionButton`, `Snackbar`…), os nomes das
props e o pacote `@azzas/ds-apps` são proposta nossa. O pacote real não foi encontrado nas fontes.

**Quem decide.** A Tecnologia.

**Texto pronto para colar:**

> Oi! Estamos testando a exportação de telas do DS Apps como código React. Para isso, montamos um mapa
> que liga cada componente do Figma ao componente de código. Os nomes abaixo são proposta nossa e
> precisamos que vocês confirmem ou corrijam:
>
> 1. **Pacote:** de onde os componentes do DS Apps são importados? Usamos `@azzas/ds-apps` como
>    exemplo.
> 2. **Nomes dos componentes:** seguimos o nome do Figma, sem acento e sem o prefixo `.Nested /`. Por
>    exemplo: `PdpProductGallery`, `PdpMainInfo`, `SizeSelectionPdp`, `ActionButton`, `Snackbar`,
>    `Button`. Esses componentes existem no código com esses nomes?
> 3. **Props:** usamos o nome da propriedade do Figma em camelCase, sem emoji e sem acento (por
>    exemplo `tamanho`, `tipo`, `icone`, `desabilitado`). É esse o padrão de vocês?
> 4. **Peças do template** (galeria, informações do produto, tamanhos, área fixa): no app, elas são
>    componentes próprios ou partes da página de produto?
>
> Se for mais fácil, podemos mandar a lista completa dos 36 componentes para vocês marcarem.

**Recomendação.** Enviar.

**Depois da resposta.** Eu corrijo o mapa (é só o arquivo de configuração no cache) e rodo a G de
novo.

---

## 7. Atualização do plugin DS

**O que é.** Durante a sessão apareceu um aviso de atualização do plugin do DS. Para usar a versão
nova, a sessão precisa ser recarregada.

**O que muda.** **Não verificado.** O aviso não trazia a lista de mudanças, e não abri a versão nova.

**Quem decide.** Você.

| Opção | A favor | Contra |
|---|---|---|
| A. Atualizar no fim de uma sessão | Pega as correções do plugin sem cortar uma frente no meio | Fica para depois |
| B. Atualizar agora | Versão nova já | Recarregar no meio de uma frente perde o contexto da sessão |
| C. Não atualizar | Nada muda | As skills do DS ficam desatualizadas |

**Recomendação.** A.

**Depois do "sim".** Você atualiza e recarrega. Na sessão seguinte, eu leio o que mudou nas skills
antes de usar.

---

## 8. Branch `eureka/t-pdp-template` com imagem de 6 MB

**O que é.** No histórico desse branch há uma imagem de 6 MB
(`demo-g3t/T1-rodada-base-template.png`, 6.007.366 bytes). A Frente T entrou no principal por squash,
só com a versão reduzida da imagem. Conferi: esse branch não está contido no principal, e a imagem
grande existe só nele. Ele existe aqui e no GitHub (`fork`).

**Quem decide.** Você. Apagar um branch não tem volta.

| Opção | A favor | Contra |
|---|---|---|
| A. Apagar aqui e no GitHub | A imagem grande some do repositório; um branch a menos | Perde o histórico de commits da T (o conteúdo já está no principal) |
| B. Manter | Nada se perde | Os 6 MB continuam no repositório |

**Recomendação.** A.

**Depois do "sim".** Eu apago o branch aqui e no GitHub e confirmo que ele sumiu.
