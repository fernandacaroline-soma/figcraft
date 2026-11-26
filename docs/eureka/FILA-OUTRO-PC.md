# Fila de tarefas entre os dois computadores

Arquivo compartilhado entre a sessão principal (orquestração; **"PC1"**) e o outro computador
(**"PC2"**). Fica no branch principal `claude/eureka-design-lead-epvzr2`. A Fernanda pediu que os
dois trabalhem juntos por aqui.

## Como usar

1. **Antes de mexer:** `git pull` no branch principal.
2. **Ao pegar uma tarefa:** troque o status de `livre` para `PC2 · <branch> · <data>`. Faça commit
   e push no branch principal só com essa linha, para o outro lado ver.
3. **Ao terminar:** status `pronto para juntar · <branch> · CI <verde|vermelho>`. O PC1 revisa e
   junta, porque a Fernanda liberou os merges em 26/11. Se a revisão pedir ajuste, o PC1 escreve
   nos **Recados**.
4. **Dúvidas, bloqueios e avisos:** escreva nos **Recados** com data e quem escreveu. Escreva
   embaixo e não apague o que o outro escreveu.
5. **Conflito no arquivo:** se o push for recusado, faça pull com rebase e repita. As alterações
   são sempre de poucas linhas.

**Regras de sempre:**
- Figma só para leitura. Nada de excluir no Figma.
- Nenhum token, componente ou regra do DS alterado.
- Nenhum conteúdo do DS no repositório. Valores ficam em `eureka-cache/`, fora do repo.
- Notion: só o card E17. Outras páginas, incluindo o Registro de propostas, pedem o ok da Fernanda.
- O repositório da Naty (gerador-propostas) não é alterado, e ninguém fala com designer.
- Para não sobrecarregar o Mac: `nice` e `-j2`, um build por vez.
- Só marque uma tarefa como concluída depois de rodar: testes, clippy, CI verde e um .md em
  `docs/eureka`.

**Arquivos do PC1:** evite mexer nestes arquivos. Se precisar, peça nos Recados.
- `crates/ui-egui/src/panels/proposal.rs`
- `crates/engine/src/cmd/eureka_explore.rs`, `eureka_critique.rs`, `eureka_screen.rs` e
  `eureka_lint.rs`
- `docs/eureka/rodada_*`, `gatekeeper_bridge.py` e `ds_components_snapshot.*`

## Tarefas

| # | Tarefa | Status |
|---|---|---|
| 1 | **Correções da revisão do export, depois o botão "Exportar código".** Detalhes em **Tarefa 1** abaixo. | juntada no principal (26/11) |
| 2 | **TSX com os componentes do DS.** Quando a instância vem de uma biblioteca do DS (`eureka.library` no componente principal), o `export.tsx` gera `<Button variant="primario">…`, importando de um pacote configurável, em vez de recriar o componente com `div` e CSS. Sem mapa de nome para código conhecido, cai no comportamento de hoje e avisa. | juntada no principal (26/11) |
| 3 | **Mapa de componente do DS para componente de código** (base do Code Connect). Um arquivo de configuração fora do repo, em `eureka-cache/`, que diz para cada conjunto do DS Apps o nome do componente de código e das props. Um comando `export.codeMap` valida o mapa contra o documento: o que falta e o que não existe. A tarefa 2 usa esse mapa. | juntada no principal (26/11) |
| 4 | **`export.tokensCss` para Web**, com Tokens WEB, Layout e a coleção de Checkout e Account, usando o snapshot `tokens-web-*` do cache. Testes com dados sintéticos. Confira se o nome das variáveis CSS bate entre App e Web, para o dev não receber dois padrões. | juntada no principal (26/11) |
| 5 | **Exportar a proposta escolhida como código.** Depois da escolha e do parecer, uma ação gera o TSX e o CSS da tela escolhida, com o parecer e a crítica num comentário de cabeçalho, como resumo para o dev. Depende da tarefa 1. Não mexa em `proposal.rs`: crie o comando e me avise nos Recados para eu ligar ao cartão. | juntada no principal (26/11) |
| 6 | **Spike TSX → Eureka**, a opção 5 que tinha ficado parada. Importar um TSX simples (o que o `export.tsx` gera) de volta como camadas, para medir a ida e volta. Só o spike, com relatório do que perde. | juntada no principal (26/11) |
| 7 | **Volta lendo os componentes do DS.** O spike da tarefa 6 lê só `div` e CSS. Faça a volta ler as chamadas `<Button variant="…" />` que o `export.tsx` gera: pelo mapa de Code Connect (o mesmo `codeConnect` ligado no documento), cada chamada vira **instância do componente do DS** com as propriedades lidas do template do mapa, e não camadas desenhadas. Chamada sem entrada no mapa: vira nota, nada de componente inventado. Testes: ida e volta de uma tela com Button e Snackbar do DS dá as mesmas instâncias e variantes; prop desconhecida vira nota. Não mexa em `proposal.rs` nem nos arquivos do PC1. | juntada no principal (26/11) |
| 8 | **Text styles no `export.tokensCss` da Web.** Hoje o CSS da Web leva cores, espaçamentos e raios, mas não os estilos de texto (família, peso, tamanho, altura de linha, espaçamento entre letras) do Tokens WEB. Gere uma classe ou um conjunto de variáveis por estilo de texto, com o mesmo padrão de nome do App (veja `docs/eureka/G3-text-styles-tokens.md`), para o dev não receber dois padrões. Testes com dados sintéticos; nenhum valor do DS no repositório. | juntada no principal (26/11) |
| 9 | **Validar a chave (E15), primeiro de tudo.** `git pull`, compile e abra o app com `--control PORTA`. Rode `python3 docs/eureka/checar_chave.py PORTA` (o script está no branch `eureka/e15-pastas` até eu juntar; depois, no principal) e cole o resultado nos Recados. Ajuste os seus scripts para mandar a chave (`eureka_control.auth(f, PORT)` logo depois do `create_connection`). Atenção: a partir do E15, os agentes só leem e gravam na pasta temporária e nas pastas que a designer autorizou em Preferências. Para ler o `eureka-cache`, abra o app com `--automation-read-root <eureka-cache> --automation-write-root <sua pasta de trabalho>`. | livre |
| 10 | **E10, teste automático: editar à mão uma instância do DS sem desanexar.** Uma instância de componente com marca de biblioteca (`eureka.library`): trocar o texto e a variante pela edição normal (os mesmos comandos dos painéis) e conferir que ela continua instância do mesmo componente, sem token novo e com origem "human". Teste no motor que falhe se a edição desanexar. | livre |
| 11 | **E10, teste automático: salvar, reabrir e manter.** Um quadro com proposta escolhida, crítica, parecer, `brand.apply` e uma edição manual: salvar em `.figcraft`, reabrir e conferir que `eureka.proposal`, `eureka.critique`, `eureka.review`, `eureka.brand`, a origem por camada e o modo da marca continuam iguais. | livre |
| 12 | **E10: o TSX exportado compila?** Um script em `docs/eureka/` que exporta a proposta G (base do template, mapa ligado) e roda `npx tsc --noEmit` num projeto React mínimo e temporário, com tipos falsos para `@azzas/ds-apps`. Relatório: compila ou não, e por quê. Fica fora do CI; não instale nada global. | livre |
| 13 | **Dívida técnica: os 3 testes do mapa mental (`board_mindmap_tests`) que falham no Mac e passam no CI.** Achar a causa (fonte do sistema? medida de texto?) e registrar em `docs/eureka/DIVIDA-TECNICA.md`. Corrigir só se a causa for clara e a correção pequena, com teste. Não mude o CI. | livre |

### Tarefa 1: detalhes

**Parte 1. Correções da revisão.** Cada uma com um teste que falha sem a correção:

1. `component_codegen.rs`, `web_with`: o `pass` das outras variantes descarta `tokens.loose`. Os
   valores soltos das variantes precisam entrar nos avisos.
2. `export_tsx.rs`, `brand_attr`: escolher o modo da coleção de marca, não o primeiro de
   `modes_for`. Avisar quando a marca vem de uma coleção de biblioteca que não tem bloco no
   `tokensCss`.
3. `css.rs`: um raio ligado só num canto, e que vale 0 na marca, perde o token.
4. `export_tsx`: ids com o mesmo nome geram componentes e imports duplicados. Deduplique ou avise.
5. `tokensCss`: coleções repetidas em `collections` não são deduplicadas.
6. Gere de novo o `docs/eureka/demo-g3/export-pdp-mobile.tsx`. O cabeçalho diz "Warnings: none",
   mas há fonte e peso soltos.

**Parte 2. Botão "Exportar código" no app.**
- Com uma tela selecionada, roda `export.tsx` e `export.tokensCss`.
- Salva os dois arquivos.
- Mostra o que veio como token e o que ficou solto (os avisos).

## Recados

- **26/11, PC1:** a pilha J, tokensCss e text styles já está no principal (`8be0fd6e`). As frentes
  O, P e Q também: crítica de design, snapshot maior e fluxo até o Gatekeeper. Agora o PC1 está
  com a Frente R, a regra 6 do lint (override de token em instância de componente oficial), em
  `eureka_lint.rs`.
- **26/11, PC2:** as tarefas 1, 4, 3 e 2 estão numa pilha, para não haver conflito em `export_tsx.rs`:
  `g3-export-code` (1) → `g3-tokens-web` (4) → `g3-tsx-ds-components` (2, com o merge de
  `g3-code-map`, a 3). O `g3-code-map` também sai direto do principal. **Juntar só o
  `g3-tsx-ds-components` leva as quatro.** O `g3-export-code` já tem o merge do principal em
  2031f220, sem conflitos. Cada tarefa tem seu .md em `docs/eureka` (G3-exportar-codigo,
  G3-tokens-web, G3-code-map, G3-tsx-componentes-ds). Atualizo o status aqui quando o CI fechar.
- **26/11, PC2 (tarefa 5):** o comando para exportar a proposta escolhida é o próximo que pego.
  Vou criá-lo sem tocar em `proposal.rs` e aviso aqui para você ligá-lo ao cartão.
- **26/11, PC2 (tarefa 5, para o PC1 ligar ao cartão):** o comando é `explore.exportCode {id, dir?}`,
  em `crates/engine/src/dev/export_handoff.rs`. Ele recusa antes do `explore.review`, gera
  `<Tela>.tsx` e `tokens.css` como o `export.code` e põe no topo do TSX o resumo da proposta, do
  parecer e dos achados da crítica ainda abertos. No cartão, depois do parecer, basta chamá-lo com
  a pasta escolhida e mostrar `tokens`/`warnings` (como em `panels/code_export.rs`). Detalhes em
  `docs/eureka/G3-handoff-proposta.md`. O branch `g3-handoff-proposta` está em cima de
  `g3-tsx-ds-components`: juntá-lo leva as tarefas 1 a 5.
- **26/11, PC1:** entendi a pilha (1 → 4 → 2 com a 3 → 5). A revisão das tarefas prontas já
  começou. Quando o CI da 2 e da 5 fechar, junto o topo (`g3-handoff-proposta`), que leva as
  cinco, e ligo o `explore.exportCode` ao cartão em `proposal.rs`, depois do parecer. Se a
  revisão pedir ajuste, escrevo aqui. Novidades no principal: frentes R e S (regras 6 e 7 do lint:
  `override-token` e `componente-local`, com `ds.fileType`), T (tela base da PDP com as peças do
  template oficial, `rodada_n.py --base=template`) e U (`explore.approve` e `explore.memory`,
  rascunho da memória visual). Agora o PC1 está com a V (contraste em todas as marcas, em
  `eureka_critique.rs`).
- **26/11, PC1 (revisão das tarefas 1 a 5):**
  - As tarefas **1 e 3 foram juntadas** no principal (`d88c4bf2`, `427b01a7`). Bom trabalho: as
    seis correções vieram com testes.
  - **Tarefa 4: ajuste pedido.** Em `export_tsx.rs`, `brand_collection` prefere a coleção local com
    mais modos à coleção de marca da biblioteca. Num arquivo comum, com a Tokens APP/WEB vinda da
    biblioteca e uma coleção local como "Layout" ou um tema de 1 modo, a local vira a marca: o TSX
    sai com `data-brand="mobile"` e a marca real vai para `data-tokens-app`. Isso desfaz a escolha
    da tarefa 1. O empate por número de modos também é decidido pela ordem. **Proposta:** a marca é
    a coleção chamada Tokens APP ou Tokens WEB, local ou de biblioteca, e só na falta delas a que
    tem mais modos. Com `collection` (prefer), o TSX e o tokensCss precisam usar a mesma coleção
    para `data-brand`; hoje o `export.code` usa sempre `brand_collection`, e os dois arquivos não
    batem. Faça um teste com coleção de biblioteca e uma local de 1 a 2 modos.
  - **Tarefa 2: ajuste pedido.** Em `export_with`/`ds_calls`, cada camada gera os próprios
    imports. Duas telas que usam Button dão `import { Button }` duas vezes no mesmo .tsx (erro de
    identificador duplicado). Deduplique os imports de todas as partes e ponha no topo, com um
    teste de duas telas. Nit: o nome de um componente importado pode bater com o nome da função de
    uma camada (raiz chamada "Button").
  - **Riscos menores**, se der:
    - Nomes "Card", "Card" e "Card2" geram duas funções `Card2` (numeração por nome original).
    - `code_map.rs` `report`: o trabalho cresce sem limite (entradas × componentes com o mesmo
      nome) e entradas duplicadas não são apontadas.
  - **Tarefa 5:** ok, depois do ajuste da 4. Um parecer "bloqueado" ainda exporta. Concordo: quem
    decide é a designer. Deixe isso escrito no cabeçalho do TSX.
  - Quando a 4, a 2 e a 5 estiverem com CI verde, marque "pronto para juntar" no topo (`g3-handoff-proposta`).
- **26/11 05:10 UTC, PC1:** a API do GitHub chegou ao limite (5.000 chamadas por hora; a conta é
  a mesma nos dois computadores) e volta às 05:26 UTC. O `gh run watch` consulta muitas vezes. Para
  acompanhar o CI, use um laço com `sleep 120` e um `gh run view` por volta, ou só confira quando
  for juntar. `git push` e `git fetch` não contam no limite.
- **26/11, PC2 (ajustes da revisão):**
  - **Tarefa 4** (`0e48c6d2`, CI verde): a marca é a coleção chamada Tokens APP/WEB, local antes da
    de biblioteca; só na falta delas, a de mais modos (local no empate). `collection` vale para o
    TSX e o tokensCss juntos. Riscos menores: numeração contra todos os nomes já dados (Card, Card2,
    Card22); `code_map` com teto de 20 000 checagens (`truncated`) e entrada duplicada em `invalid`.
  - **Tarefa 2** (`73c6f183`): os `import` de todas as telas sobem para o topo, sem repetição
    (teste de duas telas). Uma raiz com nome de componente importado vira `Button2`, com nota. O
    merge da 4 tinha perdido um `}` nos testes; corrigido.
  - **Tarefa 5** (`56170de9`): com parecer "bloqueado", o cabeçalho diz que exportou mesmo assim e
    que a designer decide. Já tem o merge da 2.
  - **Tarefa 6:** pronta, CI verde (76/77 linhas voltam iguais; detalhes em
    `docs/eureka/G3-spike-tsx-volta.md`).
  - Quando o CI da 2 e da 5 fechar verde, marco "pronto para juntar" no topo (`g3-handoff-proposta`).
- **26/11, PC1:** a tarefa 6 foi juntada (spike de ida e volta: 76 de 77 linhas iguais). A V
  (contraste em todas as marcas) também já está no principal. Aguardo o "pronto para juntar" no
  topo da pilha 1→4→2→5 (`g3-handoff-proposta`) para juntar e ligar o `explore.exportCode` ao
  cartão. Depois disso, a fila fica sem tarefas livres. Se quiser, proponha nos Recados a próxima
  do seu lado. Sugestão: a volta lendo `<Componente …/>` do DS (o que o spike não lê).
- **26/11, PC2:** CI verde na 2 (`73c6f183`) e na 5 (`56170de9`). **Pronto para juntar:**
  `eureka/g3-handoff-proposta` (leva as tarefas 1 a 5) e `eureka/g3-spike-tsx-volta` (tarefa 6).
- **26/11, PC1:** as tarefas 4, 2 e 5 foram juntadas no principal, junto com a Frente X: o botão
  "Exportar a proposta como código" no cartão chama o `explore.exportCode`. Testei na G (base do
  template): o TSX sai com o resumo, o parecer e os 11 achados abertos, mais o `tokens.css`. A fila
  está sem tarefas livres. Se quiser seguir, proponha a próxima aqui. Ideias:
  1. **Volta lendo os componentes do DS:** `<Button …/>` vira instância da biblioteca, pelo mapa.
  2. **Mapa de código das peças do template da PDP** (galeria, informações, tamanhos e área fixa).
     Hoje saem desenhadas como camadas, com 56 valores sem token na G.
  3. **`export.tokensCss` com os text styles na Web.**
- **26/11, PC1:** a Fernanda aprovou a próxima rodada. Entrou a **tarefa 7** (volta lendo os componentes do DS), livre para você. Eu sigo com a **Frente Y** (mapa de código das peças do template da PDP: galeria, informações, tamanhos e área fixa), branch `eureka/y-mapa-template`. Ela não toca o importador do spike; se você precisar do mapa, ele fica em `eureka-cache/ds-apps-pdp.connect.json`.
- **26/11, PC1:** a **Frente Y** foi juntada no principal. Com o mapa das 36 peças, a G sai sem nenhum valor sem token (antes eram 56) e com 5 chamadas. Mudanças no motor que afetam a tarefa 7: (1) o nome em código agora mantém a letra-base do acento e tira o sufixo de id do Figma ("Ícone" vira `icone`, "Texto#7:26" vira `texto`), então a volta deve ler esses nomes; (2) cada chamada do `export.tsx` traz `inside`, com o que a instância trocou por dentro (textos e variantes). A volta pode usar esse campo para reaplicar as trocas. O mapa está em `eureka-cache/ds-apps-pdp.connect.json` (só neste computador; o `docs/eureka/mapa_y.py` gera de novo). Detalhes em `docs/eureka/G3-frente-Y-mapa-template.md`.
- **26/11, PC1:** entrou a **tarefa 8** (text styles na Web), livre. A 7 também continua livre. Eu sigo com a **Frente Z** (o botão "Exportar a proposta como código" usa o mapa de código sozinho), branch `eureka/z-mapa-no-cartao`. Ela toca `proposal.rs`, `export_handoff.rs` e um comando novo em `eureka_explore.rs`; evite esses arquivos.
- **26/11, PC2 (tarefa 8, `4d183027`, CI verde):** os text styles da Web já saíam com os mesmos
  nomes do App (`--text-<estilo>-<campo>`); a regra de nome é uma só, e um teste agora garante isso.
  O erro estava no escopo: a regra do estilo era sempre `:root, [data-brand]`, então um estilo que
  lê variável do Layout não mudava com `data-layout`, e o token de peso de uma variável do Layout
  saía como `data-brand="mobile"`. Agora a regra inclui o atributo de cada coleção que o estilo lê,
  e o peso vai no atributo da própria coleção. O `collection` escolhido também chega aos text styles.
  Limites e detalhes em `docs/eureka/G3-text-styles-web.md`. Não rodei contra o snapshot real da Web.
- **26/11, PC2 (tarefa 7, `a8a55558`, CI verde):** dois comandos novos em `dev/tsx_return.rs`.
  `codeConnect.readTsx {code|path, components?}` só lê: acha o componente ligado pelo nome de código
  (e pelo pacote do import), lê as props invertendo o template do mapa e junta o `inside`.
  `codeConnect.placeCall` cria a instância com variante, props e trocas internas, num passo de
  desfazer. Chamada sem entrada no mapa e prop desconhecida viram nota; nada é inventado. O
  `spike_tsx_volta.py` usa os dois (`--in`, `--components`). Lê "Ícone"/`icone` e "Texto#7:26"/`texto`.
  Não rodei o spike de ponta a ponta no arquivo real (o mapa fica só no `eureka-cache`). Limites em
  `docs/eureka/G3-volta-componentes-ds.md`. Perguntas abertas: (1) com `--in`, o spike supõe que as
  variáveis já estão no documento; (2) com vários componentes ligados ao mesmo nome de código, ele
  escolhe a leitura com menos notas e avisa. Devia ser erro?
- **26/11, PC1 (E14, segurança):** um pedido para a **tarefa 7**. Se a volta do TSX entrar no motor (Rust), ela é código importado e precisa de limites como os outros importadores: tamanho máximo do arquivo (use `read_capped`), profundidade máxima de aninhamento (os outros usam 64 a 256), número máximo de nós e de atributos, e nenhuma execução do código lido (só parse). Testes: um arquivo grande demais, um aninhamento profundo demais e lixo são recusados sem travar; inclua o comando novo no `every_command_survives_junk_params`. Se ficar em Python, diga aqui e eu registro como risco no relatório do E14.
- **26/11, PC1:** as tarefas **7** e **8** foram revisadas e juntadas no principal. Os testes do motor passam com as duas juntas. A 7 já tinha os limites que o E14 pede para código importado (4 MB, chamadas, atributos, contagem de chaves sem estouro). Sobre as perguntas da 7: (1) supor as variáveis no documento com `--in` está ok para o spike; (2) vários componentes com o mesmo nome de código deveriam virar nota de ambiguidade, sem escolher nenhum, porque escolher em silêncio esconde um erro do mapa. Fica para uma próxima tarefa, se você quiser. A fila está sem tarefas livres. Proponha a próxima nos Recados.
- **26/11, PC1:** o **E14** (snapshots, teclado e estados do cartão, contraste AA, varredura de segredos) e o **E13** (Brand Resolver: `brand.profile` e `brand.apply`) foram juntados no principal. Novidades que afetam você: imagens de referência em `crates/engine/src/snapshots/` (para atualizar de propósito, use `FIGCRAFT_UPDATE_SNAPSHOTS=1`), um teste que varre o repositório atrás de credenciais (nunca commite chaves) e as cores novas `danger_text`/`success_text`/`warning_text` para texto de status. A fila segue sem tarefas livres; proponha a próxima nos Recados. Ideia: o `brand.apply` ler os Team Components da marca no Figma, só leitura, para mostrar as diferenças estruturais.
- **26/11, PC1 — IMPORTANTE (E15):** a conexão de controle agora exige **chave de sessão**. Depois de `git pull`, os seus scripts precisam mandar `{"method": "auth", "params": {"token": …}}` como primeira linha. Sem isso, o app recusa e fecha a conexão. Para os scripts de `docs/eureka`, já está feito, com `eureka_control.auth(f, PORT)` logo depois do `create_connection`; copie essa linha nos seus. A chave fica em `figcraft-control-<PORTA>.token`, na pasta temporária do usuário (o app mostra o caminho ao abrir), ou na variável `FIGCRAFT_CONTROL_TOKEN` se você iniciar o app com ela. O MCP (`--attach`) já manda a chave sozinho. Outra mudança: um comando com `agent` não pode mais rodar `explore.choose`, `explore.critiqueStatus`, `explore.critiqueReviewed`, `explore.approve` nem `proposal.apply`, que são decisões da designer. Nos testes que simulam a designer, chame sem `agent`. O contrato está em `docs/eureka/E15-contrato-conectores.md`.
- **26/11, PC1:** entraram as tarefas **9 a 13**. Comece pela **9**: validar a chave é o que falta para fechar o E15, por decisão da Fernanda. As tarefas 10 a 12 são os testes automáticos do roteiro do E10 (`docs/eureka/E10-roteiro-usabilidade.md`); lembre que teste automático não é validação de usabilidade, só garante que funciona. A 13 é a dívida técnica do mapa mental.
