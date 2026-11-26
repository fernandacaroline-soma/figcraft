# E14: regressão visual, acessibilidade e segurança do editor

Branch `eureka/e14-qualidade` (26/11). Card: E14 no Backlog do Eureka (Notion).

O card pede sete coisas: snapshots, teclado, contraste, estados, validação de código importado,
isolamento e segredos. O critério de aceite é: testes passando no CI e riscos críticos resolvidos
antes do Gate 2.

## Ponto de partida

Um subagente fez o inventário do que já existia, só lendo o código:

- **Já existiam:** testes de teclado do app (F6, Tab, menus), contraste do texto desabilitado,
  varredura de parâmetros inválidos em todos os comandos, limites de leitura de arquivo, sandbox de
  plugins e canal de controle só em `127.0.0.1`.
- **Faltavam:**
  - imagens de referência para comparar a renderização;
  - teclado e estados do cartão do Eureka;
  - contraste AA do texto principal e das cores de status;
  - varredura de segredos e `.env` no `.gitignore`;
  - autenticação no canal de controle.

## O que entrou

| Área | O que mudou | Teste (falha sem a mudança) |
|---|---|---|
| **Snapshots** | Uma tela de proposta montada pelo quadro do Eureka (auto layout, tokens de cor e espaço, barra fixa embaixo) é exportada em PNG nas marcas NV e Farm e comparada com imagens de referência em `crates/engine/src/snapshots/`. Sem texto, porque as fontes mudam entre máquinas. A tolerância aceita diferença de suavização de borda, não cor trocada nem bloco movido. Para atualizar as referências de propósito: `FIGCRAFT_UPDATE_SNAPSHOTS=1`. | `a_proposal_screen_draws_as_its_reference_in_each_brand`; `a_changed_token_fails_the_snapshot` prova que um token alterado é pego. |
| **Teclado** | O cartão da proposta passou a ser uma região do F6 (`Region::Card`), que só entra no ciclo enquanto aparece. O Tab circula só entre os botões do cartão, Enter e Espaço acionam, Escape volta ao canvas. Um botão que recebe o foco rola para a vista dentro do cartão. O cartão também é um contêiner com nome para leitores de tela. | `f6_reaches_the_proposal_card_and_enter_chooses_the_proposal`. Sem a região, o F6 pulava do canvas direto para as propriedades. |
| **Contraste** | Novas cores de texto de status: `danger_text`, `success_text` e `warning_text`, como o `accent_text` que já existia. O cartão do Eureka usava as cores de preenchimento como texto. No tema claro, o aviso em amarelo ficava com **1,5:1**, e no escuro o verde de "Escolhida" e "Você aprovou este layout" ficava com **2,7:1**. Agora tudo fica acima de 4,5:1. As cores de preenchimento e de ícone do app não mudaram. | `words_in_status_and_body_colours_read_at_aa_in_both_themes`: `text`, `text_secondary`, `accent_text` e as três cores de status sobre `bg`, `panel`, `field` e `hover`, nos dois temas. |
| **Estados** | Nenhuma mudança de comportamento, só a cobertura: o cartão mostra cada etapa e só a próxima ação. | `the_proposal_card_shows_each_step_and_only_the_next_action`: antes da escolha; escolhida sem crítica; crítica a revisar; revisada sem parecer; com parecer (exportar, aprovar, mapa de código); aprovada. |
| **Segredos** | `.gitignore` ganhou `.env`, `.env.*`, `*.pem`, `*.key`, `*.p12` e `id_rsa*`. Um teste varre os arquivos versionados atrás do formato de 11 tipos de credencial (chaves de API, tokens do GitHub, AWS, Slack, Google, Figma, Notion e Atlassian, blocos de chave privada). Se achar algo, mostra o arquivo e o tipo, nunca o valor. Roda com os testes do motor, então todo CI confere, sem job novo. Resultado hoje: **nenhuma credencial no repositório**. | `no_credentials_are_committed`; `the_shapes_catch_what_they_are_for`, que também confirma que nome de token do DS não é confundido com chave. |
| **Código importado** | A volta do TSX entrou no motor com a tarefa 7 do outro computador (`codeConnect.readTsx` e `codeConnect.placeCall`). Ela tem os limites pedidos: arquivo de até 4 MB, lido por `read_capped`; até 2.000 chamadas por arquivo e 200 atributos por chamada; contagem de chaves que não estoura; e o código lido nunca é executado, só lido. | Testes de lixo da própria tarefa 7, mais a varredura `every_command_survives_junk_params`, que passa por todo comando novo. |

## Riscos e o que precisa de decisão

1. **Canal de controle sem autenticação (crítico antes do 2º usuário).** O canal só aceita conexões da
   própria máquina (`127.0.0.1`) e só abre com `--control`. Mas qualquer programa local pode
   conectar e rodar qualquer comando, inclusive gravar e ler arquivos em qualquer pasta. Por exemplo,
   exportar para um caminho qualquer ou ler um mapa de código de qualquer lugar.

   A correção proposta é um token por sessão: o app gera o token ao abrir, grava num arquivo que só o
   usuário pode ler, e cada conexão precisa enviá-lo. Junto, as pastas de leitura e gravação passam a
   ser restritas por padrão quando o canal é aberto.

   Isso muda o protocolo, que é compartilhado com o FigCraft e com o outro computador. Os scripts e o
   MCP precisam passar a enviar o token. **Não foi feito: precisa da sua decisão.** O card DO-68 já
   registra que, antes do 2º usuário, é preciso reforçar a segurança da conexão local.

2. **Código importado.** A volta do TSX no motor tem limites. O script em Python que a acompanha
   (`spike_tsx_volta.py`) roda fora do app, com quem o executa, e não tem limites próprios.

3. **Bordas finas abaixo de 3:1.** As bordas dos painéis e campos ficam entre 1,1 e 1,9:1 nos dois
   temas, o que é o estilo do app. O WCAG pede 3:1 para o contorno de controles que precisam ser
   identificados. Mudar isso muda o visual do app inteiro: é decisão de design.

4. **Cores de status no resto do app.** Fora do cartão do Eureka, cerca de 40 lugares ainda usam as
   cores de preenchimento como texto. As cores novas de texto estão prontas para eles. A troca fica
   para quando esses painéis forem mexidos, porque é código do FigCraft original.

5. **CI do Eureka.** O job `eureka` não roda os testes de integração do motor nem os doctests, e o
   job `full` é só consultivo. Mudar isso é mudar o CI, o que as regras do repositório não permitem
   num branch de funcionalidade.

## Validação

- Testes novos: 7. Cada um falha sem a mudança. O de teclado foi conferido removendo a região: o F6
  ia para "Properties" em vez de "Card".
- `cargo clippy -D warnings` no motor e na interface: limpo.
- Motor: 2164 testes passam. Interface: 1235 passam.
- Neste Mac também falham, sem esta mudança: 3 testes do mapa mental e 1 de área de transferência.
  No CI, eles passam.
- Imagens de referência: `crates/engine/src/snapshots/eureka-pdp-nv.png` e `eureka-pdp-farm.png`,
  geradas e conferidas a olho. São sintéticas, só blocos de cor, sem conteúdo do DS.
- Evidência no app: `docs/eureka/demo-e14/` (cartão no tema claro com as cores novas).
