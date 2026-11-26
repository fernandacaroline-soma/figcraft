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
| 1 | **Correções da revisão do export, depois o botão "Exportar código".** Detalhes em **Tarefa 1** abaixo. | PC2 (passada em 25/11) |
| 2 | **TSX com os componentes do DS.** Quando a instância vem de uma biblioteca do DS (`eureka.library` no componente principal), o `export.tsx` gera `<Button variant="primario">…`, importando de um pacote configurável, em vez de recriar o componente com `div` e CSS. Sem mapa de nome para código conhecido, cai no comportamento de hoje e avisa. | livre |
| 3 | **Mapa de componente do DS para componente de código** (base do Code Connect). Um arquivo de configuração fora do repo, em `eureka-cache/`, que diz para cada conjunto do DS Apps o nome do componente de código e das props. Um comando `export.codeMap` valida o mapa contra o documento: o que falta e o que não existe. A tarefa 2 usa esse mapa. | livre |
| 4 | **`export.tokensCss` para Web**, com Tokens WEB, Layout e a coleção de Checkout e Account, usando o snapshot `tokens-web-*` do cache. Testes com dados sintéticos. Confira se o nome das variáveis CSS bate entre App e Web, para o dev não receber dois padrões. | livre |
| 5 | **Exportar a proposta escolhida como código.** Depois da escolha e do parecer, uma ação gera o TSX e o CSS da tela escolhida, com o parecer e a crítica num comentário de cabeçalho, como resumo para o dev. Depende da tarefa 1. Não mexa em `proposal.rs`: crie o comando e me avise nos Recados para eu ligar ao cartão. | livre |
| 6 | **Spike TSX → Eureka**, a opção 5 que tinha ficado parada. Importar um TSX simples (o que o `export.tsx` gera) de volta como camadas, para medir a ida e volta. Só o spike, com relatório do que perde. | livre |

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
