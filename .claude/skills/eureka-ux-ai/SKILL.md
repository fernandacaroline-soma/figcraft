---
name: eureka-ux-ai
description: UX e interação com IA do Eureka Design (card E06, "ai-interaction-design"). Use sempre que for projetar, revisar, prototipar ou alterar qualquer experiência visual do Eureka (painéis, canvas, fluxos com Claude, conectores de agentes, aprovações), ou quando alguém pedir "proposta de interação", "protótipo de fluxo com IA", "como o designer chama o Claude", "revisão de UX", "estados da IA". Garante edição visual como centro, IA com escopo, preview, diff por item e desfazer, proteção de edições manuais, origem IA × humano, governança do Design System e decisão humana nos gates.
---

# Eureka UX & AI Interaction

Versão v1 (13/11/2026, card E06). Substitui o rascunho v0 da Sprint 0.
Dona das decisões de produto e UX: a liderança de Design do Eureka.
Esta skill orienta propostas. Ela não aprova nenhuma direção de UX sozinha.

## 1. Direção aprovada (combinar, não reabrir)

- **A · Pedido na seleção** para ajustes rápidos. Selecionar → atalho → barra junto da seleção com
  escopo explícito → execução visível e cancelável → preview fantasma no canvas.
- **C · Quadro de exploração** para problemas abertos. Hipótese → 3 a 5 alternativas lado a lado com
  evidências → escolha humana → crítica de Design → DS Gatekeeper → refino.
- **Cartão de proposta (de B)** é a superfície ÚNICA de revisão para A e C. Diff legível por item,
  aceitar parcial, parecer do Gatekeeper, Aceitar / Ajustar / Rejeitar. Aceitar = 1 passo de desfazer.

Nova proposta de UX que cria uma segunda superfície de revisão, ou que troca A/C por chat, precisa
de decisão explícita da dona.

## 2. Princípios (verifique cada proposta contra todos)

1. **O canvas é o centro.** Edição visual direta é o caminho principal. Nenhum ajuste simples (cor,
   espaço, texto, ordem) exige prompt. Ex.: valor solto tem botão "Vincular ao token" no painel Design.
2. **A IA vê o que o designer fez.** Antes de cada pedido, o estado atual do documento é a fonte,
   incluindo edições manuais. A IA nunca regenera por cima. Ela diz quantas edições leu.
3. **Escopo explícito.** Mostre sobre o que a IA vai agir (seleção, frame pai, página) antes de rodar.
   Durante a execução, o escopo fica congelado e visível. Fora do escopo, ela pede permissão item por item.
4. **Proposta antes de aplicar.** Preview fantasma no canvas, diff por item no cartão, aceitar parcial.
   Aceitar = 1 desfazer. Rejeitar não deixa rastro no documento.
5. **Execução visível e interrompível.** Passos nomeados, onde a IA age, Cancelar com Esc que funciona.
   Cancelar e erro sempre dizem "nada foi alterado" (ou o que foi).
6. **Edição manual vence por padrão.** Se o designer mexe numa camada durante a execução ou depois da
   proposta, o item vira conflito. O padrão é manter a edição dele. Trocar exige escolha explícita.
7. **Origem IA × humano por camada.** Camada gerada ou alterada pela IA fica marcada até o designer
   editá-la. Daí em diante é "Você" e a IA preserva.
8. **Desfazer em dois níveis.** ⌘Z desfaz o último passo (inclui o aceite inteiro). "Desfazer só a IA"
   reverte o que a IA aplicou e mantém edições manuais feitas depois.
9. **Design System primeiro, checado antes da revisão humana.** Gere com tokens, componentes e
   variantes oficiais. Valor solto é sinalizado, nunca escondido. O pré-parecer do Gatekeeper aparece no
   cartão antes do designer decidir e separa "já existia" de "introduzido pela proposta". A IA não cria
   nem altera tokens, componentes ou regras globais: item que faria isso vem **bloqueado** com
   "Encaminhar à governança".
10. **Marca com fonte.** Só perfis de marca preenchidos (responsável + data). Sem perfil, diga a
    limitação e trabalhe com usabilidade e DS.
11. **Agentes independentes, Eureka como cockpit.** Gatekeeper, Builder, Docs, Audit e o gerador de
    propostas da Naty rodam fora. O Eureka chama, mostra estado e resultado, e pede decisão. Agente
    fora do ar vira estado visível ("indisponível · tentar de novo"), não silêncio.
12. **Humano decide nos gates.** Escolha entre alternativas, registro no Notion, envio ao Figma,
    exclusões, merge e mudança de DS pedem confirmação explícita. Foco padrão no "Cancelar".
13. **Não interrompa.** Sugestões não abrem modal nem roubam foco. Se o designer está digitando quando a
    proposta chega, anuncie e não mova o foco.

## 3. O que o benchmark (E03) ensina

Fonte: `docs/eureka/E03-benchmark-ux-ia.md` (fontes oficiais acessadas em 13/11/2026). Números entre
colchetes são as fontes do E03.

**Fazem bem (copiar o padrão, não a interface):**
- Selecionar e pedir é o padrão dominante (Lovable, Bolt, Cursor, Figma Make, v0) [9][12][15][23]. → A.
- Dois caminhos: controle direto para propriedade, IA para estrutura (v0, Bolt, Figma Make) [5][9][15].
  → princípio 1.
- Edições pendentes antes de aplicar: v0 Apply com antes/depois [9], Bolt lote revisável [15], Figma
  Make em stage [5]. → princípio 4 e toggle "Prévia no canvas".
- Variações lado a lado com comparação ao original e Keep/Discard (Subframe) [16][17]. → C com card
  "Original".
- Esc para a resposta e pedidos em fila (Subframe) [16]. → Cancelar com Esc.
- Acessibilidade concreta só no Claude Design: leitor de tela, F6 entre regiões, contraste no seletor
  de cor [3]. → é o parâmetro mínimo do Eureka.

**Fazem mal (evitar):**
- Reversão só por versão do projeto inteiro (Lovable diz que não há reversão parcial [14]; Bolt não
  desfaz item salvo [15]).
- Cmd+Z que não desfaz o agente (Cursor [23]; Subframe para MCP/API [16]).
- IA que altera vizinhos fora do pedido (relato em v0 [10], fonte secundária).
- Mudança aparece só no fim, sem preview, no caminho por prompt (Bolt [15]).
- Chat como única interface (Penpot via MCP externo [26]; Stitch [19]).

**Lacunas confirmadas (o diferencial do Eureka):**

| Lacuna | O que o Eureka faz |
|---|---|
| Origem por camada IA × humano: nenhuma ferramenta documenta | Marca "IA" / "Você" por camada no painel Camadas e no canvas (princípio 7) |
| Proteção de ajustes manuais quando a IA refaz: nenhuma documenta | Conflito por item, padrão = manter a edição manual (princípio 6) |
| Desfazer só a IA sem perder edições posteriores: nenhuma tem | "Desfazer só a IA" por proposta aceita, com relatório do que foi mantido (princípio 8) |
| Checagem de DS antes da revisão humana: DS é insumo em algumas (Cursor [24], Builder [28], Claude Design [3]), mas ninguém valida | Pré-parecer do Gatekeeper dentro do cartão, recalculado a cada item marcado (princípio 9) |

Ressalva do E03: "não encontrado" = ausente nas páginas oficiais lidas, não prova de inexistência.

## 4. Estados obrigatórios

Toda superfície de IA (barra A, quadro C, cartão) cobre estes estados. Cada um diz o que aconteceu com
o documento.

| Estado | O designer vê | Regra |
|---|---|---|
| **Vazio** | Nada selecionado: "selecione uma camada ou use a página inteira" | Nunca assumir a página inteira em silêncio |
| **Escopo** | Chip "Escopo: Button + 1 camada", opções Seleção / Frame pai / Página, contorno no canvas | Visível antes de enviar |
| **Executando** | Passos nomeados, progresso, escopo congelado, Cancelar (Esc) | Designer pode seguir editando |
| **Cancelado** | "Parei no passo 2 de 5. Nada foi alterado." + Pedir de novo | Pedido preservado no campo |
| **Erro** | Passo que falhou, causa em linguagem de designer, "nada foi alterado", ação (Tentar de novo, Copiar detalhe) | Sem desculpa vaga, sem JSON |
| **Sem resultado** | "Nenhuma mudança proposta" + o que a IA entendeu | Não inventar mudança para ter o que mostrar |
| **Proposta pronta** | Cartão com diff por item, prévia no canvas, pré-parecer do Gatekeeper | Nada aplicado ainda |
| **Agente indisponível** | "Gatekeeper indisponível · tentar de novo" | Revisão possível, mas sem selo falso |
| **Aplicado** | Aviso "Aplicado: n itens em 1 passo · Desfazer" + camadas marcadas "IA" | 1 entrada no histórico de desfazer |
| **Desfeito** | "Desfeito: …" ou "Desfiz só a IA; mantive suas edições em X" | Dizer o que foi mantido |
| **Conflito** | Item "você mudou isto durante a execução": Manter a sua (padrão) / Usar a do Claude | Vale também para edição feita depois da proposta chegar |
| **Bloqueado** | Item que tocaria componente oficial ou token: desabilitado + "Encaminhar à governança" | Nunca aplicável pela IA |

## 5. Bom × ruim

1. **Escopo**
   - Bom: "Escopo: Button + 1 camada dentro". Selo pedido fora dele vem desmarcado, com "fora do escopo, marque para permitir".
   - Ruim: pedir "adicionar selo" com o botão selecionado e a IA mexer no card inteiro sem avisar.
2. **Edição manual durante a execução**
   - Bom: o designer troca a cor do Label enquanto o Claude roda. O cartão mostra conflito, já marcado em "Manter a sua".
   - Ruim: a proposta chega e aplica a cor do Claude por cima, porque leu o estado no início.
3. **Valor solto**
   - Bom: painel Design mostra `#7A4B2A` como "valor solto · fora do DS" e oferece "Vincular a Colors/Surface/Brand/Background".
   - Ruim: a IA troca o hex pelo token em silêncio, ou esconde o hex atrás de um nome parecido com token.
4. **Diff**
   - Bom: "Button · Cor: `#7A4B2A` (valor solto) → `Colors/Surface/Brand/Background`", com checkbox e realce da camada ao passar o mouse.
   - Ruim: "3 alterações aplicadas" ou um JSON de comandos.
5. **Desfazer**
   - Bom: um ⌘Z desfaz o aceite inteiro. "Desfazer só a IA" mantém a cor que o designer mudou depois.
   - Ruim: um ⌘Z por passo interno da IA, ou reverter a versão do projeto inteiro.
6. **Erro**
   - Bom: "Falhou no passo 3: não consegui ler as variáveis do modo Marca Exemplo. Nada foi alterado. Tentar de novo."
   - Ruim: "Algo deu errado." ou `engine.execute failed: {...}`.
7. **Gatekeeper**
   - Bom: pré-parecer no cartão, antes de aceitar, separando "já existia" de "introduzido pela proposta"; muda ao desmarcar itens.
   - Ruim: selo "OK" fixo, ou checagem só depois de aplicar.
8. **Exploração**
   - Bom: 4 alternativas lado a lado com o Original, chips de evidência, pré-parecer por alternativa; escolher abre o mesmo cartão.
   - Ruim: alternativas no chat em texto, ou uma tela de revisão diferente para cada modo.
9. **Gates externos**
   - Bom: "Registrar estudo no Notion?" e "Enviar a escolhida ao Figma?" com confirmação e foco em Cancelar.
   - Ruim: registrar no Notion ou mandar ao Figma automaticamente depois de aceitar.
10. **Foco**
    - Bom: proposta chega enquanto o designer digita no painel Design; o status anuncia e o foco fica onde está.
    - Ruim: modal da proposta rouba o foco no meio da edição.

## 6. Checklist de uma proposta de UX

- [ ] Fluxo do ponto de vista do designer, passo a passo.
- [ ] Como a IA é chamada (atalho, seleção, botão) e de onde vem o contexto.
- [ ] Escopo visível antes de enviar e congelado durante a execução.
- [ ] Todos os estados da seção 4, com o que acontece com o documento.
- [ ] Diff por item, aceitar parcial, prévia no canvas ligável.
- [ ] Edições manuais durante a execução e depois da proposta: conflito com padrão "manter a sua".
- [ ] Origem IA × humano visível por camada.
- [ ] Desfazer: 1 passo para o aceite; desfazer só a IA.
- [ ] Gatekeeper: pré-parecer antes da decisão humana; o que bloqueia; estado indisponível.
- [ ] Gates humanos (Notion, Figma, governança) com confirmação.
- [ ] Acessibilidade: teclado completo, foco visível, `aria-live` para status da IA, contraste AA, `prefers-reduced-motion`.
- [ ] Protótipo clicável ou screenshot do app anexado.
- [ ] Riscos e decisões que ficam com a dona.

## 7. Anti-padrões

- Interface que só funciona por chat.
- Regenerar a tela inteira para mudar um detalhe.
- Aplicar sem preview, ou com um desfazer por passo interno.
- Sobrescrever edição manual sem perguntar, mesmo para "corrigir" para o DS.
- Esconder valor fora do DS ou inventar token.
- Alterar componente oficial ou token pela IA.
- Inventar característica de marca.
- Modal para sugestão.
- Segunda superfície de revisão além do cartão.

## 8. Teste da skill (critério de aceite do E06)

Aplicada em duas tarefas em 13/11/2026. ✅ atende · ⚠️ atende em parte · ❌ não atende.

### Tarefa 1 · Protótipo interativo da direção A + C + cartão

Arquivo: protótipo HTML de página única (publicado como página privada para a dona testar).
Testado no navegador em desktop e em 375 px, temas claro e escuro, por script de cliques e screenshots.
Não testado com leitor de tela real nem com designers.

| Item | Resultado | Evidência |
|---|---|---|
| Fluxo do designer | ✅ | Roteiro de 5 passos no topo da página |
| Chamada da IA | ✅ | ⌘K / Ctrl+K, "/" e botão "Pedir ao Claude"; barra abre junto da seleção (ancorada abaixo da camada; no celular entra no fluxo) |
| Escopo | ✅ | Chip + Seleção / Frame pai / Página; contorno no canvas; "escopo congelado" ao executar; selo fora do escopo vem desmarcado |
| Estados | ✅ | Vazio, escopo, executando, cancelado (Esc, "parei no passo 2 de 5, nada foi alterado"), erro (cenário "Falha na execução"), sem resultado, proposta, Gatekeeper indisponível, aplicado, desfeito, conflito, bloqueado |
| Diff e aceitar parcial | ✅ | Checkbox por item; botão vira "Aceitar 2 de 3"; hover/foco realça a camada; prévia fantasma ligável |
| Edição manual durante a execução | ✅ | Trocar a cor do Label durante a execução gerou item de conflito com "Manter a sua" marcado; edição em outra propriedade vira nota "preservei" |
| Origem IA × humano | ⚠️ | Por camada, não por propriedade. Uma camada com cor da IA e texto do designer aparece só como "Você" |
| Desfazer | ✅ | ⌘Z desfaz o aceite inteiro; "Desfazer só a IA" removeu selo e respiro e manteve a cor que o designer mudou depois. Não há refazer |
| Gatekeeper | ⚠️ | Pré-parecer no cartão, recalculado por item, com contraste AA e "já existia × introduzido". É simulado; no quadro C o pré-parecer de cada alternativa é fixo |
| Gates humanos | ✅ | Notion e Figma com diálogo de confirmação, foco em Cancelar; componente oficial bloqueado com "Encaminhar à governança" |
| Acessibilidade | ⚠️ | Teclado em tudo, setas nas camadas, foco visível, `aria-live` e `role=alert`, `prefers-reduced-motion`, tokens do editor medidos entre 6,0:1 e 17,7:1. Falha: nós do canvas não recebem foco (seleção por teclado só pelo painel Camadas). Sem teste com leitor de tela |
| Protótipo anexado | ✅ | Página clicável |
| Riscos e decisões | ✅ | Listados no relatório à dona |

Falhas honestas: "Ajustar" entende só palavras-chave (cores, selo, respiro, minúsculas). As
alternativas do quadro C são fixas, não dependem da hipótese. Sem multisseleção. No desktop o cartão
ocupa a direita do canvas. Valores de cor são sintéticos.

### Tarefa 2 · Revisão crítica do spike real (`docs/eureka/spike_hybrid_loop.py`)

O spike roda o ciclo gerar → editar → ler → refinar pelo canal de controle do FigCraft. É um script de
desenvolvedor, não uma UI. Por isso a revisão olha o **comportamento** que ele modela.

| Item da skill | Resultado | Evidência no spike |
|---|---|---|
| DS primeiro (cores) | ✅ | Card e refino usam `{"$var": ...}` para todas as cores |
| DS primeiro (espaço e tipo) | ❌ | `padding` 16/12, `gap` 16/8, `fontSize` 14/16/13/11 e `strokeWeight` em número solto, embora o spike importe os tokens de espaçamento. Nada sinaliza isso |
| A IA vê o que o designer fez | ⚠️ | O passo `read` lê texto, cor e vínculos, mas por nome de camada fixo (`ids.json`), não por um delta "o que mudou". Uma camada nova do designer passaria despercebida |
| Preserva edição manual | ⚠️ | Preserva os textos. Mas **sobrescreve sem perguntar** as cores que o designer escolheu à mão (`#7A4B2A` e `#FFFFFF` voltam para token). Pela skill isso é item de conflito ou proposta, não correção silenciosa |
| Proposta antes de aplicar | ❌ | `refine` aplica direto. Não há preview, diff nem aceitar parcial (lacuna já registrada no handoff) |
| Escopo explícito | ❌ | O refino age em ids fixos e ainda adiciona um selo que nenhum pedido no fluxo nomeia |
| Aceitar = 1 desfazer | ✅ | Refino em um `command.batch` com rótulo; um ⌘Z desfaz tudo |
| Origem IA × humano | ⚠️ | Lotes do Claude levam `agent="Claude"`; edições manuais não. O dado existe por lote, mas não vira marca por camada |
| Execução visível e cancelável | ❌ | Sem progresso nem cancelamento; o script bloqueia até terminar |
| Erro claro | ❌ | Falha vira `SystemExit` com JSON cortado em 600 caracteres |
| Gatekeeper antes da revisão | ❌ | Não é chamado |
| Edição "manual" real | ⚠️ | O passo `manual` usa `node.set` pelo script, não o mouse. O S1-1 do plano já prevê isso |

Conclusão: o spike prova o motor (tokens, leitura de estado, 1 desfazer). Ele **viola** os princípios
4, 5, 6 e 9 e trata edição manual como erro a corrigir. Para a Sprint 1, os pontos mais caros de
corrigir depois são: proposta antes de aplicar (S1-2), delta de edição (S1-3) e conflito com edição
manual como regra do motor, não da UI.

## 9. Fluxo de exploração (preservar)

Problema/hipótese → 3 a 5 propostas com evidências (benchmarking, usabilidade, inovação) → escolha
humana → crítica de Design → DS Gatekeeper → refino no cartão de proposta. O Notion guarda o estudo e
todas as alternativas. O Figma recebe só a escolhida. Os dois com confirmação humana.
