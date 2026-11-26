# Frente Q: o fluxo inteiro da exploração, do quadro ao Gatekeeper

Branch `eureka/q-fluxo-gatekeeper` (25 e 26/11). Faz parte do E17 (G3) e vem depois da Frente P.

## O que foi feito

Pela primeira vez, a sequência da skill gerador-propostas rodou inteira no Eureka, com o parecer
real do ds-gatekeeper no fim:

1. **Quadro.** A Rodada 01 da NV, com 8 propostas: as 5 incrementais e as 3 disruptivas de teste da
   Frente P. Teve lint 0.
2. **Escolha.** A proposta G, "Comprar agora no tamanho de sempre", foi escolhida como teste. A
   escolha real é da designer.
3. **Crítica.** A crítica teve 11 achados, 6 medidos pelo Eureka e 5 do roteiro. A revisão da
   designer foi simulada, também como teste.
4. **Pedido ao Gatekeeper.** O `gatekeeper_bridge.py export-choice` montou o pedido. Ele só funciona
   com a crítica já revisada e leva tudo junto: proposta, lint, contexto e crítica
   (`Q0-pedido-gatekeeper.json`).
5. **Parecer.** O ds-gatekeeper respondeu em modo prévia (só leitura, sem registro no Notion), com
   **"com ressalvas"** e 6 itens (`Q2-parecer-gatekeeper.json`). O `post-choice` gravou o parecer
   na proposta.
6. **Cartão.** O cartão mostra o parecer (`Q1`).

## O parecer do Gatekeeper sobre a G

- **Dois botões na área de ação fixa da PDP.** As instâncias são oficiais, mas o par de botões muda
  o padrão de botão principal único em todas as marcas do DS Apps. Se não houver regra escrita, é um
  padrão novo, e a decisão é da liderança de DS Apps.
- **Snackbar dentro do conteúdo.** Pede uma decisão: usar a posição documentada ou levar o uso
  dentro do conteúdo à liderança. Também pede a variação de erro, que a crítica já tinha apontado.
- **Galeria, nome e preço como camadas simples.** Pede verificar se existem componentes oficiais
  equivalentes, ou marcar essas camadas como placeholder de exploração.
- **Estado incoerente.** O botão dizia "Comprar agora · M", mas nenhum tamanho estava selecionado.
  **Corrigido:** o `rodada_n.py` aceita `picked`, e a G mostra o M selecionado. O parecer gravado
  na demo é anterior a essa correção.
- **Textos literais.** Pergunta se o botão principal e a mensagem da sacola usam token de texto por
  marca. Fica em aberto.
- **Limite do próprio parecer.** É uma prévia sem diff, sem acesso ao Docusaurus e sem checagem de
  override nas instâncias. Não vale como pré-aprovação técnica.

Os itens 1 e 2 são decisões da liderança de DS. Nada foi alterado no DS.

## O que mudou no app

**No cartão, a crítica revisada fica recolhida.** Antes, o parecer do Gatekeeper ficava no fim do
cartão, depois dos 11 achados, e só aparecia rolando. Agora, depois de "Revisei a crítica", o cartão
mostra:

- o resumo da crítica (contagem por severidade e "Você revisou a crítica");
- **"Ver os achados"** recolhido, que abre a lista completa com o status de cada achado;
- o parecer logo abaixo, à vista.

Antes da revisão, a crítica continua aberta, porque é ela que a designer precisa ler.

## Limites

- **Testes no fluxo:** a escolha e a revisão da crítica foram feitas como teste. A G é uma proposta
  de teste com conteúdo de exemplo.
- **Itens 3 e 5 do parecer:** a galeria e os textos continuam como estão. São decisões de produto
  e de DS, não do Eureka.
- **Gatekeeper sem acesso ao DS:** o parecer é feito só sobre o pedido, sem diff e sem o
  Docusaurus. Uma pré-aprovação formal pede a revisão completa.

## Rodar

```
python3 docs/eureka/rodada_n.py W eureka-cache/tokens-app-animale-farm-nv-b86217b8e421.json eureka-cache/components-app-frente-p-295d7a197126.json docs/eureka/rodada_nv01_propostas.json docs/eureka/rodada_nv01_disruptivas.json
python3 docs/eureka/rodada_o.py W <id da G> docs/eureka/rodada_nv01_critica_G.json
python3 docs/eureka/gatekeeper_bridge.py export-choice W <id da G>    # → W/gk-request.json
# o ds-gatekeeper (Claude, só leitura) lê o pedido e responde com o JSON do parecer
python3 docs/eureka/gatekeeper_bridge.py post-choice W <id da G> parecer.json
```
